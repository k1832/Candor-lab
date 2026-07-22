/* GENERATED C mirror of reference module m150. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S150_0;

static S150_0 mk150_0(long a) {
    S150_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe150_0(const S150_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read150_0(const S150_0 *s) {
    return s->a * 6;
}
static void bump150_0(S150_0 *s, long d) {
    s->a = s->a + d;
}
static long classify150_0(int tag, long a, long b) {
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
static long accum150_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard150_0(long x) {
    return x + 8;
}

static long pick150_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S150_1;

static S150_1 mk150_1(long a) {
    S150_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe150_1(const S150_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read150_1(const S150_1 *s) {
    return s->a * 2;
}
static void bump150_1(S150_1 *s, long d) {
    s->a = s->a + d;
}
static long classify150_1(int tag, long a, long b) {
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
static long accum150_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard150_1(long x) {
    return x + 2;
}

static long pick150_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S150_2;

static S150_2 mk150_2(long a) {
    S150_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe150_2(const S150_2 *s) {
    return s->a + s->n0;
}
static long read150_2(const S150_2 *s) {
    return s->a * 6;
}
static void bump150_2(S150_2 *s, long d) {
    s->a = s->a + d;
}
static long classify150_2(int tag, long a, long b) {
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
static long accum150_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard150_2(long x) {
    return x + 4;
}

static long pick150_2_0(long a, long b) { return a > b ? a : b; }
long f150(long x) {
    long acc = x;
    acc += f055(x + 1);
    acc += f096(x + 2);
    S150_0 s0 = mk150_0(acc);
    bump150_0(&s0, 9);
    acc += probe150_0(&s0);
    acc += read150_0(&s0);
    acc += classify150_0(1, acc, acc);
    acc += accum150_0(7);
    acc += guard150_0(acc);
    acc += pick150_0_0(acc, acc + 2);
    S150_1 s1 = mk150_1(acc);
    bump150_1(&s1, 4);
    acc += probe150_1(&s1);
    acc += read150_1(&s1);
    acc += classify150_1(1, acc, acc);
    acc += accum150_1(8);
    acc += guard150_1(acc);
    acc += pick150_1_0(acc, acc + 9);
    S150_2 s2 = mk150_2(acc);
    bump150_2(&s2, 6);
    acc += probe150_2(&s2);
    acc += read150_2(&s2);
    acc += classify150_2(1, acc, acc);
    acc += accum150_2(8);
    acc += guard150_2(acc);
    acc += pick150_2_0(acc, acc + 7);
    return clampi(acc);
}
