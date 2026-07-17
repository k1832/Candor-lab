/* Candor FREESTANDING runtime (design 0010 §5; LANG_PHILOSOPHY P7/P9/NN#6).
 *
 * The no-libc twin of `aot_runtime.c`. It links the SAME emitted Candor object
 * (imports the six `rt_*` shims, exports `candor_entry`) but depends on NO libc
 * and no OS-facing runtime: the ONLY host services it uses are two raw Linux
 * syscalls — `write` (for the θ-trace and the fault line) and `exit` — issued
 * directly, never through a C library. This proves NN#6's "no mandatory runtime;
 * freestanding targets are first-class": the payload runs with `-nostdlib`.
 *
 * HONEST SCOPE: freestanding here means NO LIBC, not no-kernel. `_start` still
 * asks the x86-64 Linux kernel to `write`/`exit` via `syscall`. True bare-metal
 * (no kernel, MMIO console, qemu-system) is deferred — that is a different target.
 *
 * THE FLAT MEMORY REGION is a static .bss-style NOBITS section placed at the
 * fixed VA `MEM_BASE` by the linker (`--section-start`), so the compile-time
 * `MEM_BASE + candor_addr` constant the lowering bakes resolves with no mmap and
 * no disk cost (NOBITS stores no bytes in the ELF; the kernel zero-fills it).
 *
 * THE FAULT PATH is the minimal second point on P7's fault-policy axis for a
 * freestanding root: NO setjmp/longjmp. The root fault handler (`rt_fault`) is
 * called DIRECTLY at the fault site; the default freestanding policy is HALT —
 * write a minimal `(kind, span)` fault line and `exit(2)`. No unwinding: Cranelift
 * frames carry no destructors, and halting needs none.
 *
 * Built by `cc -ffreestanding -nostdlib -static -no-pie`.
 */
#include <stdint.h>

/* Must match src/interp/mem.rs and src/backend/object.rs MEM_BASE. */
#define MEM_BASE    0x0000200000000000UL
#define MAX_ADDR    0x10000000UL   /* 256 MiB */
#define STACK_BASE  0x00100000UL

/* The flat memory region: a NOBITS section pinned to MEM_BASE by the linker
 * (`-Wl,--section-start=.candor_flat=0x200000000000`). No mmap; the kernel maps
 * this LOAD segment zero-filled at startup. */
__asm__(
  ".section .candor_flat,\"aw\",@nobits\n"
  ".globl candor_flat_region\n"
  ".p2align 12\n"
  "candor_flat_region: .skip 0x10000000\n"
  ".previous\n"
);

/* Address the region only through the absolute MEM_BASE constant (a movabs), never
 * the far symbol: a -no-pie PC-relative reference to a 32 TiB VA does not fit. */
static uint8_t *const g_base = (uint8_t *)MEM_BASE;
static uint64_t g_stack_bump = STACK_BASE;

/* ---- raw Linux/x86-64 syscalls (the ONLY host dependency) ---------------- */
static long sys_write(long fd, const void *buf, unsigned long n) {
    long r;
    __asm__ volatile("syscall" : "=a"(r)
                     : "a"(1L), "D"(fd), "S"(buf), "d"(n)
                     : "rcx", "r11", "memory");
    return r;
}
static __attribute__((noreturn)) void sys_exit(int code) {
    __asm__ volatile("syscall" :: "a"(60L), "D"((long)code) : "rcx", "r11");
    __builtin_unreachable();
}

/* ---- freestanding libc primitives (no libc to link) ---------------------- */
/* Defined so both our code AND any implicit compiler/codegen calls resolve. */
void *memset(void *d, int c, unsigned long n) {
    uint8_t *p = (uint8_t *)d;
    for (unsigned long i = 0; i < n; i++) p[i] = (uint8_t)c;
    return d;
}
void *memcpy(void *d, const void *s, unsigned long n) {
    uint8_t *pd = (uint8_t *)d; const uint8_t *ps = (const uint8_t *)s;
    for (unsigned long i = 0; i < n; i++) pd[i] = ps[i];
    return d;
}
void *memmove(void *d, const void *s, unsigned long n) {
    uint8_t *pd = (uint8_t *)d; const uint8_t *ps = (const uint8_t *)s;
    if (pd < ps) { for (unsigned long i = 0; i < n; i++) pd[i] = ps[i]; }
    else { for (unsigned long i = n; i > 0; i--) pd[i - 1] = ps[i - 1]; }
    return d;
}

static uint64_t round_up(uint64_t x, uint64_t a) {
    if (a < 1) a = 1;
    return ((x + a - 1) / a) * a;
}

/* Write a signed decimal + '\n' to fd (θ-trace and fault-line integers). */
static void write_decln(long fd, int64_t v) {
    char buf[24];
    int i = (int)sizeof(buf);
    buf[--i] = '\n';
    uint64_t u = v < 0 ? (uint64_t)(-(v + 1)) + 1ULL : (uint64_t)v; /* -MIN safe */
    if (u == 0) { buf[--i] = '0'; }
    else { while (u) { buf[--i] = (char)('0' + (u % 10)); u /= 10; } }
    if (v < 0) buf[--i] = '-';
    sys_write(fd, &buf[i], (unsigned long)((int)sizeof(buf) - i));
}
static void write_str(long fd, const char *s) {
    unsigned long n = 0;
    while (s[n]) n++;
    sys_write(fd, s, n);
}
/* Space-separated decimal (no newline): the fault line's span numbers. */
static void write_dec(long fd, uint32_t v) {
    char buf[16]; int i = (int)sizeof(buf);
    if (v == 0) { buf[--i] = '0'; }
    else { while (v) { buf[--i] = (char)('0' + (v % 10)); v /= 10; } }
    sys_write(fd, &buf[i], (unsigned long)((int)sizeof(buf) - i));
}

/* ---- the six rt_* shims the Candor object imports ------------------------ */
uint64_t rt_stack_alloc(uint64_t size, uint64_t align) {
    uint64_t a = round_up(g_stack_bump, align < 1 ? 1 : align);
    g_stack_bump = a + size;
    if (size) memset(g_base + a, 0, size);
    return a;
}
void rt_copy(uint64_t dst, uint64_t src, uint64_t len) {
    if (len) memmove(g_base + dst, g_base + src, len);
}
/* The observable θ-trace hook: raw write syscall to stdout, one decimal per line
 * (INV-OBS-ORDER — a syscall barrier; identical bytes to the hosted printf). */
void rt_trace(int64_t v) { write_decln(1, v); }
int64_t rt_mmio_load(uint64_t addr, uint64_t size) {
    uint8_t b[8] = {0}; uint64_t n = size < 8 ? size : 8;
    memcpy(b, g_base + addr, n);
    int64_t r; memcpy(&r, b, 8); return r;
}
void rt_mmio_store(uint64_t addr, int64_t val, uint64_t size) {
    uint64_t n = size < 8 ? size : 8;
    memcpy(g_base + addr, &val, n);
}

/* FaultKind numeric code (backend::lower::kind_code) -> snake_case name. */
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

/* The root fault handler, called DIRECTLY (no setjmp). Freestanding policy: HALT.
 * Minimal fault line to stderr, then exit(2) — the P7 halt-and-log second point.
 * Format: `candor fault: <kind> <start> <end>` (kind + span, the gate parses both). */
void rt_fault(uint32_t kind, uint32_t s, uint32_t e) {
    write_str(2, "candor fault: ");
    write_str(2, kind_name(kind));
    write_str(2, " ");
    write_dec(2, s);
    write_str(2, " ");
    write_dec(2, e);
    write_str(2, "\n");
    sys_exit(2);
}

/* The compiler-emitted startup glue: writes string bytes, runs static inits, calls
 * main, returns its i64. */
extern int64_t candor_entry(void);

/* Process entry (no crt, no __libc_start_main): run the payload, exit with main's
 * low byte (Unix convention). θ has already streamed to stdout via rt_trace. */
__attribute__((noreturn)) void _start(void) {
    int64_t r = candor_entry();
    sys_exit((int)(uint8_t)(uint64_t)r);
}
