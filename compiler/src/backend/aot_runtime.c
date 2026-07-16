/* Candor AOT runtime (design 0010 §5, Stage B's cranelift-object note; design
 * 0012 Stage 2 for the concurrency hooks).
 *
 * The tiny static runtime the emitted object links against — the AOT twin of
 * `src/backend/runtime.rs` (the JIT's host shim). It owns the flat memory model,
 * the (atomic) stack bump allocator, the observable `trace`/MMIO hooks, the
 * fault-exit hook, and — new in Stage 2 — REAL OS-thread `spawn`/join over raw
 * pthreads. It is the process entry: it maps the flat buffer, runs the compiler-
 * emitted `candor_entry` inside a `setjmp` landing pad, and translates the result
 * to the process exit protocol — EXACTLY the CLI `run` contract (fault => exit 2
 * plus the (kind, span) JSON on stderr) so the differential harness can compare.
 *
 * Built by `cc -pthread` at compile time (the least-dependency path: no second
 * Rust staticlib, `cc` is already required for linking). Freestanding of the
 * Candor object except for the `rt_*` symbols and `candor_entry`.
 *
 * ## Runtime-internal synchronization (BELOW the language, design 0012 §1.3 note)
 * The runtime's own structures carry their own synchronization, mirroring
 * runtime.rs: the stack-bump pointer is a C11 atomic (a CAS-bumped allocator,
 * giving each concurrent frame a disjoint region), and the trace sink, fault slot,
 * and open-scope stack are PER-THREAD (`__thread`) — each task accumulates into its
 * own buffers, merged deterministically at the join (spawn order => deterministic
 * θ). The flat buffer stays unsynchronized; the Stage-1 checker's DRF guarantee
 * makes concurrent disjoint/read-only access sound.
 *
 * ## The fault-exit hook, per-thread (design 0012 §6)
 * Every MIR fault edge lowers to `call rt_fault(kind, s, e)`. The hook records
 * `(k, s)` into THIS thread's fault slot and `longjmp`s to THIS thread's landing
 * pad — the main thread's is `thread_main`'s, a task thread's is `task_start`'s.
 * A cross-thread `longjmp` would be undefined, so each task catches its own fault
 * locally and reports it as an outcome; the join (`rt_scope_end`) re-delivers the
 * spawn-order-first fault (§3.2) on the parent thread.
 */
#include <stddef.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <setjmp.h>
#include <stdatomic.h>
#include <sys/mman.h>
#include <pthread.h>
#include <dirent.h>
#include <sys/socket.h>
#include <netdb.h>
#include <unistd.h>

/* Must match src/interp/mem.rs and src/backend/object.rs MEM_BASE. */
#define MEM_BASE    0x0000200000000000UL
#define MAX_ADDR    0x10000000UL   /* 256 MiB */
#define STACK_BASE  0x00100000UL

/* The fixed number of marshalled i64 arg slots `rt_spawn` receives (mirrors
 * `backend::lower::MAX_SPAWN_ARGS`). */
#define MAX_SPAWN_ARGS 6

static uint8_t *g_base;
static atomic_uint_fast64_t g_stack_bump;

extern int64_t candor_entry(void);

static uint64_t round_up(uint64_t x, uint64_t a) {
    if (a < 1) a = 1;
    return ((x + a - 1) / a) * a;
}

/* Reserve + zero a stack slot; return its Candor address. A CAS-bumped atomic so
 * concurrent task threads each get a disjoint region (runtime-internal sync,
 * design 0012 §1.3 note); the bump never rolls back, so live frames stay
 * disjoint by construction (mirrors runtime.rs `rt_stack_alloc`). */
uint64_t rt_stack_alloc(uint64_t size, uint64_t align) {
    if (align < 1) align = 1;
    for (;;) {
        uint64_t cur = atomic_load_explicit(&g_stack_bump, memory_order_relaxed);
        uint64_t a = round_up(cur, align);
        uint64_t next = a + size;
        if (atomic_compare_exchange_weak_explicit(
                &g_stack_bump, &cur, next,
                memory_order_seq_cst, memory_order_relaxed)) {
            if (size) memset(g_base + a, 0, (size_t)size);
            return a;
        }
    }
}

/* Byte-copy `len` bytes src -> dst within the flat model. */
void rt_copy(uint64_t dst, uint64_t src, uint64_t len) {
    if (len) memmove(g_base + dst, g_base + src, (size_t)len);
}

/* ------------------------------------------------------------------------- */
/* std::io directory listing (design 0013 dir listing over the 0011 boundary). */
/* ------------------------------------------------------------------------- */

/* `sys_listdir` binds here (there is no libc `listdir`): opendir the
 * NUL-terminated `path`, enumerate entries EXCLUDING "." and ".." (matching the
 * interpreter shim's `std::fs::read_dir`, which omits them), and — only when
 * `dcap` is large enough — write each name followed by a NUL into `dst`. Returns
 * the total bytes the NUL-separated names need (the two-call sizing datum), or -1
 * on error. `path`/`dst` arrive already translated to real host pointers at the
 * boundary (backend::lower::host_addr). The entry SET matches the interpreter
 * shim; the readdir order may differ (the test sorts). */
int64_t listdir(const char *path, char *dst, uint64_t dcap) {
    DIR *d = opendir(path);
    if (!d) return -1;
    uint64_t needed = 0;
    struct dirent *e;
    while ((e = readdir(d)) != NULL) {
        const char *n = e->d_name;
        if (n[0] == '.' && (n[1] == '\0' || (n[1] == '.' && n[2] == '\0'))) continue;
        needed += (uint64_t)strlen(n) + 1;
    }
    if (dcap >= needed && needed > 0) {
        rewinddir(d);
        uint64_t off = 0;
        while ((e = readdir(d)) != NULL) {
            const char *n = e->d_name;
            if (n[0] == '.' && (n[1] == '\0' || (n[1] == '.' && n[2] == '\0'))) continue;
            uint64_t len = (uint64_t)strlen(n);
            memcpy(dst + off, n, (size_t)len);
            off += len;
            dst[off++] = '\0';
        }
    }
    closedir(d);
    return (int64_t)needed;
}

/* ------------------------------------------------------------------------- */
/* std::net TCP client (design 0013 std net over the 0011 boundary).         */
/* ------------------------------------------------------------------------- */

/* `sys_tcp_connect` binds here (there is no libc `tcp_connect`): resolve the
 * `host_len`-byte `host` (numeric like "127.0.0.1" or a name) + `port` via
 * getaddrinfo, then socket/connect the first address that accepts. Returns the
 * connected fd (>= 0) — an ORDINARY descriptor, so libc read/write/close (the
 * sys_read/sys_write/sys_close externs) drive send/recv/close — or -1 on failure
 * (unresolvable host, or every address refused/unreachable). `host` arrives already
 * translated to a real host pointer at the boundary (backend::lower::host_addr) and
 * is NOT NUL-terminated, so it is copied into a bounded local first. */
int64_t tcp_connect(const char *host, uint64_t host_len, int32_t port) {
    char hbuf[256];
    if (host_len >= sizeof(hbuf)) return -1;
    memcpy(hbuf, host, (size_t)host_len);
    hbuf[host_len] = '\0';

    char pbuf[16];
    snprintf(pbuf, sizeof(pbuf), "%d", (int)port);

    struct addrinfo hints;
    memset(&hints, 0, sizeof(hints));
    hints.ai_family = AF_UNSPEC;
    hints.ai_socktype = SOCK_STREAM;

    struct addrinfo *res = NULL;
    if (getaddrinfo(hbuf, pbuf, &hints, &res) != 0) return -1;

    int fd = -1;
    for (struct addrinfo *ai = res; ai != NULL; ai = ai->ai_next) {
        fd = socket(ai->ai_family, ai->ai_socktype, ai->ai_protocol);
        if (fd < 0) continue;
        if (connect(fd, ai->ai_addr, ai->ai_addrlen) == 0) break;
        close(fd);
        fd = -1;
    }
    freeaddrinfo(res);
    return (int64_t)fd;
}

/* ------------------------------------------------------------------------- */
/* Per-thread (thread-local) runtime state (design 0012 §6).                 */
/* ------------------------------------------------------------------------- */

typedef struct { uint32_t kind, s, e; int has; } RtFault;

/* A growable i64 trace buffer (this thread's `θ` fragment). */
typedef struct {
    int64_t *data;
    size_t   len, cap;
} TraceBuf;

static void trace_push(TraceBuf *tb, int64_t v) {
    if (tb->len == tb->cap) {
        tb->cap = tb->cap ? tb->cap * 2 : 16;
        tb->data = realloc(tb->data, tb->cap * sizeof(int64_t));
    }
    tb->data[tb->len++] = v;
}

/* One spawned task: its marshalled call + its OS thread handle. */
typedef struct {
    uint64_t  faddr;
    int       argc;
    int64_t   args[MAX_SPAWN_ARGS];
    pthread_t thread;
} Task;

/* An open `scope` frame: the tasks spawned into it, joined in spawn order. */
typedef struct {
    Task **tasks;
    size_t len, cap;
} ScopeFrame;

/* What a joined task hands back to its parent: its caught fault (if any) and its
 * own trace fragment (merged into the parent's in spawn order). */
typedef struct {
    RtFault  fault;
    TraceBuf trace;
} TaskOutcome;

typedef struct {
    jmp_buf    *land;   /* this thread's active landing pad (main or task) */
    RtFault     fault;  /* the fault caught on this thread */
    TraceBuf    trace;  /* this thread's θ fragment; merged at each join */
    ScopeFrame *scopes; /* stack of open scope frames (nested `scope`s) */
    size_t      slen, scap;
} Tls;

static __thread Tls *g_tls;

static Tls *tls(void) {
    if (!g_tls) g_tls = calloc(1, sizeof(Tls));
    return g_tls;
}

/* The observable trace(x) hook: append to THIS task's buffer (per-task
 * projection). The join merges buffers in spawn order, so θ is
 * schedule-independent (design 0012 §6). */
void rt_trace(int64_t v) {
    trace_push(&tls()->trace, v);
}

/* The observable rawptr/MMIO load hook: read `size` bytes, zero-extended. */
int64_t rt_mmio_load(uint64_t addr, uint64_t size) {
    uint8_t buf[8] = {0};
    uint64_t n = size < 8 ? size : 8;
    memcpy(buf, g_base + addr, (size_t)n);
    int64_t r;
    memcpy(&r, buf, 8);
    return r;
}

/* The observable rawptr/MMIO store hook: write the low `size` bytes of `val`. */
void rt_mmio_store(uint64_t addr, int64_t val, uint64_t size) {
    uint64_t n = size < 8 ? size : 8;
    memcpy(g_base + addr, &val, (size_t)n);
}

/* The fault-exit hook: record (kind, span) in THIS thread's fault slot and
 * longjmp to THIS thread's landing pad (never a cross-thread jump). */
void rt_fault(uint32_t kind, uint32_t s, uint32_t e) {
    Tls *t = tls();
    t->fault.kind = kind;
    t->fault.s = s;
    t->fault.e = e;
    t->fault.has = 1;
    longjmp(*t->land, 1);
}

/* Dispatch a compiled task fn (`extern "C" fn(i64, ...) -> i64`) by arity. Every
 * Candor arg — scalar or a pointer to caller-owned aggregate storage — is one i64
 * in the backend ABI, so arity alone selects the signature (mirrors
 * runtime.rs `call_task`). */
static void call_task(uint64_t faddr, int argc, const int64_t *a) {
    uintptr_t p = (uintptr_t)faddr;
    switch (argc) {
        case 0: ((int64_t (*)(void))p)(); break;
        case 1: ((int64_t (*)(int64_t))p)(a[0]); break;
        case 2: ((int64_t (*)(int64_t, int64_t))p)(a[0], a[1]); break;
        case 3: ((int64_t (*)(int64_t, int64_t, int64_t))p)(a[0], a[1], a[2]); break;
        case 4: ((int64_t (*)(int64_t, int64_t, int64_t, int64_t))p)(a[0], a[1], a[2], a[3]); break;
        case 5: ((int64_t (*)(int64_t, int64_t, int64_t, int64_t, int64_t))p)(a[0], a[1], a[2], a[3], a[4]); break;
        case 6: ((int64_t (*)(int64_t, int64_t, int64_t, int64_t, int64_t, int64_t))p)(a[0], a[1], a[2], a[3], a[4], a[5]); break;
        default:
            fprintf(stderr, "rt_spawn: task arity %d exceeds MAX_SPAWN_ARGS\n", argc);
            exit(3);
    }
}

/* The task-thread body: establish this thread's own fault landing pad, run the
 * task fn with its marshalled args, catch any fault locally, and hand back the
 * (fault, trace) outcome (never longjmp-ing across the thread boundary). Nested-
 * scope children of this task have already merged their traces into `trace` at
 * their own rt_scope_end. */
static void *task_start(void *arg) {
    Task *tk = (Task *)arg;
    Tls *t = tls();
    jmp_buf jb;
    t->land = &jb;
    TaskOutcome *out = calloc(1, sizeof(TaskOutcome));
    if (setjmp(jb) == 0) {
        call_task(tk->faddr, tk->argc, tk->args);
    }
    out->fault = t->fault;
    out->trace = t->trace; /* hand over the buffer (ownership moves to `out`) */
    t->trace.data = NULL;
    t->trace.len = t->trace.cap = 0;
    return out;
}

/* The opening `{` of a `scope`: push a fresh frame onto this thread's scope
 * stack (design 0012 §1.1). */
void rt_scope_begin(void) {
    Tls *t = tls();
    if (t->slen == t->scap) {
        t->scap = t->scap ? t->scap * 2 : 4;
        t->scopes = realloc(t->scopes, t->scap * sizeof(ScopeFrame));
    }
    ScopeFrame *fr = &t->scopes[t->slen++];
    fr->tasks = NULL;
    fr->len = 0;
    fr->cap = 0;
}

/* `spawn CALLEE(args)`: create a REAL OS thread running the task fn at `faddr`
 * with `argc` marshalled i64 args, and record it in the innermost open scope
 * frame (joined at the closing brace, in spawn order). The generous per-task host
 * stack matches the JIT (native recursion inside a task); the Candor "stack" lives
 * in the flat buffer via rt_stack_alloc. */
void rt_spawn(int64_t faddr, int64_t argc,
              int64_t a0, int64_t a1, int64_t a2,
              int64_t a3, int64_t a4, int64_t a5) {
    Tls *t = tls();
    Task *tk = calloc(1, sizeof(Task));
    tk->faddr = (uint64_t)faddr;
    tk->argc = (int)argc;
    int64_t src[MAX_SPAWN_ARGS] = {a0, a1, a2, a3, a4, a5};
    memcpy(tk->args, src, sizeof(src));

    pthread_attr_t attr;
    pthread_attr_init(&attr);
    pthread_attr_setstacksize(&attr, 64UL * 1024 * 1024);
    if (pthread_create(&tk->thread, &attr, task_start, tk) != 0) {
        fprintf(stderr, "rt_spawn: could not create task thread\n");
        exit(3);
    }
    pthread_attr_destroy(&attr);

    ScopeFrame *fr = &t->scopes[t->slen - 1];
    if (fr->len == fr->cap) {
        fr->cap = fr->cap ? fr->cap * 2 : 4;
        fr->tasks = realloc(fr->tasks, fr->cap * sizeof(Task *));
    }
    fr->tasks[fr->len++] = tk;
}

/* The closing `}` / join barrier: join every task of the innermost scope frame in
 * SPAWN ORDER, merge their per-task traces into this thread's trace (deterministic
 * θ, regardless of fault extent), and deliver the SPAWN-ORDER-FIRST fault (§3.2) —
 * recorded in this thread's slot and longjmp-ed to this thread's landing pad — if
 * any task faulted. */
void rt_scope_end(void) {
    Tls *t = tls();
    ScopeFrame fr = t->scopes[--t->slen];
    RtFault first;
    first.has = 0;
    for (size_t i = 0; i < fr.len; i++) {
        Task *tk = fr.tasks[i];
        void *ret = NULL;
        pthread_join(tk->thread, &ret);
        TaskOutcome *o = (TaskOutcome *)ret;
        for (size_t j = 0; j < o->trace.len; j++) {
            trace_push(&t->trace, o->trace.data[j]);
        }
        if (!first.has && o->fault.has) {
            first = o->fault;
        }
        free(o->trace.data);
        free(o);
        free(tk);
    }
    free(fr.tasks);
    if (first.has) {
        t->fault = first;
        longjmp(*t->land, 1);
    }
}

/* FaultKind numeric code (backend::lower::kind_code) -> serde snake_case name,
 * so the stderr JSON matches the CLI's `Fault::to_json`. */
static const char *kind_name(uint32_t k) {
    switch (k) {
        case 0: return "overflow";
        case 1: return "div_by_zero";
        case 2: return "bounds";
        case 3: return "conv_loss";
        case 4: return "assert";
        case 5: return "requires";
        case 6: return "ensures";
        case 7: return "panic";
        case 8: return "bad_pointer";
        default: return "no_foreign_runtime";
    }
}

static int      g_is_fault;
static int64_t  g_ret;
static RtFault  g_main_fault;
static TraceBuf g_main_trace;

/* Runs on a large-stack thread: native recursion uses the host stack (the
 * interpreter used heap frames), so give it the JIT's 512 MiB reach. The setjmp
 * must live in the thread that runs candor_entry for longjmp to unwind it. The
 * main thread's merged trace already carries every joined task's fragments. */
static void *thread_main(void *arg) {
    (void)arg;
    Tls *t = tls();
    jmp_buf jb;
    t->land = &jb;
    if (setjmp(jb)) {
        g_is_fault = 1;
        g_main_fault = t->fault;
    } else {
        g_is_fault = 0;
        g_ret = candor_entry();
    }
    g_main_trace = t->trace;
    t->trace.data = NULL;
    t->trace.len = t->trace.cap = 0;
    return NULL;
}

int main(void) {
    void *p = mmap((void *)MEM_BASE, MAX_ADDR, PROT_READ | PROT_WRITE,
                   MAP_PRIVATE | MAP_ANONYMOUS | MAP_FIXED, -1, 0);
    if (p == MAP_FAILED || p != (void *)MEM_BASE) {
        fprintf(stderr, "candor runtime: could not map the flat memory region\n");
        return 3;
    }
    g_base = (uint8_t *)p;
    atomic_store_explicit(&g_stack_bump, STACK_BASE, memory_order_relaxed);

    pthread_attr_t attr;
    pthread_attr_init(&attr);
    pthread_attr_setstacksize(&attr, 512UL * 1024 * 1024);
    pthread_t tmain;
    if (pthread_create(&tmain, &attr, thread_main, NULL) != 0) {
        fprintf(stderr, "candor runtime: could not spawn the execution thread\n");
        return 3;
    }
    pthread_join(tmain, NULL);
    pthread_attr_destroy(&attr);

    /* θ: the deterministically merged trace, streamed one decimal per line. */
    for (size_t i = 0; i < g_main_trace.len; i++) {
        printf("%lld\n", (long long)g_main_trace.data[i]);
    }
    fflush(stdout);

    if (g_is_fault) {
        const char *k = kind_name(g_main_fault.kind);
        fprintf(stderr,
                "{\"kind\":\"%s\",\"span\":{\"start\":%u,\"end\":%u},\"message\":\"%s\"}\n",
                k, g_main_fault.s, g_main_fault.e, k);
        return 2;
    }
    /* Success: main's i64 result mapped to the process exit code (low byte,
     * Unix convention). */
    return (int)(uint8_t)(uint64_t)g_ret;
}
