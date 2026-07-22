/* GENERATED C mirror of reference module m100. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S100_0;

static S100_0 mk100_0(long a) {
    S100_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe100_0(const S100_0 *s) {
    return s->a + s->n0;
}
static long read100_0(const S100_0 *s) {
    return s->a * 6;
}
static void bump100_0(S100_0 *s, long d) {
    s->a = s->a + d;
}
static long classify100_0(int tag, long a, long b) {
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
static long accum100_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard100_0(long x) {
    return x + 2;
}

static long pick100_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S100_1;

static S100_1 mk100_1(long a) {
    S100_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe100_1(const S100_1 *s) {
    return s->a + s->n0;
}
static long read100_1(const S100_1 *s) {
    return s->a * 2;
}
static void bump100_1(S100_1 *s, long d) {
    s->a = s->a + d;
}
static long classify100_1(int tag, long a, long b) {
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
static long accum100_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard100_1(long x) {
    return x + 2;
}

static long pick100_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S100_2;

static S100_2 mk100_2(long a) {
    S100_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe100_2(const S100_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read100_2(const S100_2 *s) {
    return s->a * 6;
}
static void bump100_2(S100_2 *s, long d) {
    s->a = s->a + d;
}
static long classify100_2(int tag, long a, long b) {
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
static long accum100_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard100_2(long x) {
    return x + 3;
}

static long pick100_2_0(long a, long b) { return a > b ? a : b; }
static long pick100_2_1(long a, long b) { return a > b ? a : b; }
long f100(long x) {
    long acc = x;
    acc += f016(x + 1);
    acc += f069(x + 2);
    S100_0 s0 = mk100_0(acc);
    bump100_0(&s0, 3);
    acc += probe100_0(&s0);
    acc += read100_0(&s0);
    acc += classify100_0(1, acc, acc);
    acc += accum100_0(7);
    acc += guard100_0(acc);
    acc += pick100_0_0(acc, acc + 6);
    S100_1 s1 = mk100_1(acc);
    bump100_1(&s1, 4);
    acc += probe100_1(&s1);
    acc += read100_1(&s1);
    acc += classify100_1(1, acc, acc);
    acc += accum100_1(7);
    acc += guard100_1(acc);
    acc += pick100_1_0(acc, acc + 9);
    S100_2 s2 = mk100_2(acc);
    bump100_2(&s2, 3);
    acc += probe100_2(&s2);
    acc += read100_2(&s2);
    acc += classify100_2(1, acc, acc);
    acc += accum100_2(8);
    acc += guard100_2(acc);
    acc += pick100_2_0(acc, acc + 2);
    acc += pick100_2_1(acc, acc + 8);
    return clampi(acc);
}
