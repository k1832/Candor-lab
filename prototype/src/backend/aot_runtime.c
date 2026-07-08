/* Candor AOT runtime (design 0010 §5, Stage B's cranelift-object note).
 *
 * The tiny static runtime the emitted object links against — the AOT twin of
 * `src/backend/runtime.rs` (the JIT's host shim). It owns the flat memory model,
 * the stack bump allocator, the observable `trace`/MMIO hooks, and the fault-exit
 * hook, and it is the process entry: it maps the flat buffer, runs the compiler-
 * emitted `candor_entry` inside a `setjmp` landing pad, and translates the result
 * to the process exit protocol — EXACTLY the CLI `run` contract (fault => exit 2
 * plus the (kind, span) JSON on stderr) so the differential harness can compare.
 *
 * Built by `cc` at compile time (the least-dependency path: no second Rust
 * staticlib, `cc` is already required for linking). Freestanding of the Candor
 * object except for the six `rt_*` symbols and `candor_entry`.
 */
#include <stddef.h>
#include <stdint.h>
#include <string.h>
#include <stdio.h>
#include <setjmp.h>
#include <sys/mman.h>
#include <pthread.h>

/* Must match src/interp/mem.rs and src/backend/object.rs MEM_BASE. */
#define MEM_BASE    0x0000200000000000UL
#define MAX_ADDR    0x10000000UL   /* 256 MiB */
#define STACK_BASE  0x00100000UL

static uint8_t *g_base;
static uint64_t g_stack_bump;
static jmp_buf  g_jmp;
static uint32_t g_fault_kind, g_fault_start, g_fault_end;

extern int64_t candor_entry(void);

static uint64_t round_up(uint64_t x, uint64_t a) {
    if (a < 1) a = 1;
    return ((x + a - 1) / a) * a;
}

/* Reserve + zero a stack slot; return its Candor address (mirrors
 * Mem::stack_alloc + zero-on-alloc). */
uint64_t rt_stack_alloc(uint64_t size, uint64_t align) {
    uint64_t a = round_up(g_stack_bump, align < 1 ? 1 : align);
    g_stack_bump = a + size;
    if (size) memset(g_base + a, 0, (size_t)size);
    return a;
}

/* Byte-copy `len` bytes src -> dst within the flat model. */
void rt_copy(uint64_t dst, uint64_t src, uint64_t len) {
    if (len) memmove(g_base + dst, g_base + src, (size_t)len);
}

/* The observable trace(x) hook: θ prints to stdout, one decimal per line. */
void rt_trace(int64_t v) {
    printf("%lld\n", (long long)v);
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

/* The fault-exit hook: record (kind, span) and longjmp to the landing pad. */
void rt_fault(uint32_t kind, uint32_t s, uint32_t e) {
    g_fault_kind = kind;
    g_fault_start = s;
    g_fault_end = e;
    longjmp(g_jmp, 1);
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

static int    g_is_fault;
static int64_t g_ret;

/* Runs on a large-stack thread: native recursion uses the host stack (the
 * interpreter used heap frames), so give it the JIT's 512 MiB reach. The setjmp
 * must live in the thread that runs candor_entry for longjmp to unwind it. */
static void *thread_main(void *arg) {
    (void)arg;
    if (setjmp(g_jmp)) {
        g_is_fault = 1;
        return NULL;
    }
    g_is_fault = 0;
    g_ret = candor_entry();
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
    g_stack_bump = STACK_BASE;

    pthread_attr_t attr;
    pthread_attr_init(&attr);
    pthread_attr_setstacksize(&attr, 512UL * 1024 * 1024);
    pthread_t t;
    if (pthread_create(&t, &attr, thread_main, NULL) != 0) {
        fprintf(stderr, "candor runtime: could not spawn the execution thread\n");
        return 3;
    }
    pthread_join(t, NULL);
    pthread_attr_destroy(&attr);

    fflush(stdout);
    if (g_is_fault) {
        const char *k = kind_name(g_fault_kind);
        fprintf(stderr,
                "{\"kind\":\"%s\",\"span\":{\"start\":%u,\"end\":%u},\"message\":\"%s\"}\n",
                k, g_fault_start, g_fault_end, k);
        return 2;
    }
    /* Success: main's i64 result mapped to the process exit code (low byte,
     * Unix convention). θ has already streamed to stdout via rt_trace. */
    return (int)(uint8_t)(uint64_t)g_ret;
}
