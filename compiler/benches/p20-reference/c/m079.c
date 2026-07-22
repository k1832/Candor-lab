/* GENERATED C mirror of reference module m079. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S79_0;

static S79_0 mk79_0(long a) {
    S79_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe79_0(const S79_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read79_0(const S79_0 *s) {
    return s->a * 4;
}
static void bump79_0(S79_0 *s, long d) {
    s->a = s->a + d;
}
static long classify79_0(int tag, long a, long b) {
    switch (tag) {
    case 0:
        return 0;
    case 1:
        return a;
    case 2:
        return a + b;
    default:
        return a * 2;
    }
}
static long accum79_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard79_0(long x) {
    return x + 5;
}

static long pick79_0_0(long a, long b) { return a > b ? a : b; }
static long pick79_0_1(long a, long b) { return a > b ? a : b; }
static long pick79_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S79_1;

static S79_1 mk79_1(long a) {
    S79_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe79_1(const S79_1 *s) {
    return s->a + s->n0;
}
static long read79_1(const S79_1 *s) {
    return s->a * 5;
}
static void bump79_1(S79_1 *s, long d) {
    s->a = s->a + d;
}
static long classify79_1(int tag, long a, long b) {
    switch (tag) {
    case 0:
        return 0;
    case 1:
        return a;
    case 2:
        return a + b;
    default:
        return a * 2;
    }
}
static long accum79_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard79_1(long x) {
    return x + 9;
}

static long pick79_1_0(long a, long b) { return a > b ? a : b; }
static long pick79_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S79_2;

static S79_2 mk79_2(long a) {
    S79_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe79_2(const S79_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read79_2(const S79_2 *s) {
    return s->a * 3;
}
static void bump79_2(S79_2 *s, long d) {
    s->a = s->a + d;
}
static long classify79_2(int tag, long a, long b) {
    switch (tag) {
    case 0:
        return 0;
    case 1:
        return a;
    case 2:
        return a + b;
    default:
        return a * 2;
    }
}
static long accum79_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard79_2(long x) {
    return x + 2;
}

static long pick79_2_0(long a, long b) { return a > b ? a : b; }
static long pick79_2_1(long a, long b) { return a > b ? a : b; }
static long pick79_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S79_3;

static S79_3 mk79_3(long a) {
    S79_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe79_3(const S79_3 *s) {
    return s->a + s->n0;
}
static long read79_3(const S79_3 *s) {
    return s->a * 6;
}
static void bump79_3(S79_3 *s, long d) {
    s->a = s->a + d;
}
static long classify79_3(int tag, long a, long b) {
    switch (tag) {
    case 0:
        return 0;
    case 1:
        return a;
    case 2:
        return a + b;
    default:
        return a * 2;
    }
}
static long accum79_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard79_3(long x) {
    return x + 9;
}

static long pick79_3_0(long a, long b) { return a > b ? a : b; }
long f079(long x) {
    long acc = x;
    acc += f045(x + 1);
    acc += f076(x + 2);
    S79_0 s0 = mk79_0(acc);
    bump79_0(&s0, 1);
    acc += probe79_0(&s0);
    acc += read79_0(&s0);
    acc += classify79_0(1, acc, acc);
    acc += accum79_0(7);
    acc += guard79_0(acc);
    acc += pick79_0_0(acc, acc + 8);
    acc += pick79_0_1(acc, acc + 7);
    acc += pick79_0_2(acc, acc + 6);
    S79_1 s1 = mk79_1(acc);
    bump79_1(&s1, 4);
    acc += probe79_1(&s1);
    acc += read79_1(&s1);
    acc += classify79_1(1, acc, acc);
    acc += accum79_1(5);
    acc += guard79_1(acc);
    acc += pick79_1_0(acc, acc + 6);
    acc += pick79_1_1(acc, acc + 2);
    S79_2 s2 = mk79_2(acc);
    bump79_2(&s2, 6);
    acc += probe79_2(&s2);
    acc += read79_2(&s2);
    acc += classify79_2(1, acc, acc);
    acc += accum79_2(3);
    acc += guard79_2(acc);
    acc += pick79_2_0(acc, acc + 7);
    acc += pick79_2_1(acc, acc + 6);
    acc += pick79_2_2(acc, acc + 8);
    S79_3 s3 = mk79_3(acc);
    bump79_3(&s3, 9);
    acc += probe79_3(&s3);
    acc += read79_3(&s3);
    acc += classify79_3(1, acc, acc);
    acc += accum79_3(6);
    acc += guard79_3(acc);
    acc += pick79_3_0(acc, acc + 5);
    return clampi(acc);
}
