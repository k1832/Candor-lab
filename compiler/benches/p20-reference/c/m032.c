/* GENERATED C mirror of reference module m032. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S32_0;

static S32_0 mk32_0(long a) {
    S32_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe32_0(const S32_0 *s) {
    return s->a + s->n0;
}
static long read32_0(const S32_0 *s) {
    return s->a * 7;
}
static void bump32_0(S32_0 *s, long d) {
    s->a = s->a + d;
}
static long classify32_0(int tag, long a, long b) {
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
static long accum32_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard32_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S32_1;

static S32_1 mk32_1(long a) {
    S32_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe32_1(const S32_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read32_1(const S32_1 *s) {
    return s->a * 2;
}
static void bump32_1(S32_1 *s, long d) {
    s->a = s->a + d;
}
static long classify32_1(int tag, long a, long b) {
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
static long accum32_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard32_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S32_2;

static S32_2 mk32_2(long a) {
    S32_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe32_2(const S32_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read32_2(const S32_2 *s) {
    return s->a * 6;
}
static void bump32_2(S32_2 *s, long d) {
    s->a = s->a + d;
}
static long classify32_2(int tag, long a, long b) {
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
static long accum32_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard32_2(long x) {
    return x + 8;
}

long f032(long x) {
    long acc = x;
    acc += f011(x + 1);
    acc += f019(x + 2);
    acc += f021(x + 3);
    S32_0 s0 = mk32_0(acc);
    bump32_0(&s0, 6);
    acc += probe32_0(&s0);
    acc += read32_0(&s0);
    acc += classify32_0(1, acc, acc);
    acc += accum32_0(7);
    acc += guard32_0(acc);
    S32_1 s1 = mk32_1(acc);
    bump32_1(&s1, 1);
    acc += probe32_1(&s1);
    acc += read32_1(&s1);
    acc += classify32_1(1, acc, acc);
    acc += accum32_1(7);
    acc += guard32_1(acc);
    S32_2 s2 = mk32_2(acc);
    bump32_2(&s2, 4);
    acc += probe32_2(&s2);
    acc += read32_2(&s2);
    acc += classify32_2(1, acc, acc);
    acc += accum32_2(9);
    acc += guard32_2(acc);
    return clampi(acc);
}
