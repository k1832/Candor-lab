/* GENERATED C mirror of reference module m016. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S16_0;

static S16_0 mk16_0(long a) {
    S16_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe16_0(const S16_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read16_0(const S16_0 *s) {
    return s->a * 5;
}
static void bump16_0(S16_0 *s, long d) {
    s->a = s->a + d;
}
static long classify16_0(int tag, long a, long b) {
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
static long accum16_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard16_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S16_1;

static S16_1 mk16_1(long a) {
    S16_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe16_1(const S16_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read16_1(const S16_1 *s) {
    return s->a * 2;
}
static void bump16_1(S16_1 *s, long d) {
    s->a = s->a + d;
}
static long classify16_1(int tag, long a, long b) {
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
static long accum16_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard16_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S16_2;

static S16_2 mk16_2(long a) {
    S16_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe16_2(const S16_2 *s) {
    return s->a + s->n0;
}
static long read16_2(const S16_2 *s) {
    return s->a * 2;
}
static void bump16_2(S16_2 *s, long d) {
    s->a = s->a + d;
}
static long classify16_2(int tag, long a, long b) {
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
static long accum16_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard16_2(long x) {
    return x + 9;
}

long f016(long x) {
    long acc = x;
    acc += f004(x + 1);
    S16_0 s0 = mk16_0(acc);
    bump16_0(&s0, 7);
    acc += probe16_0(&s0);
    acc += read16_0(&s0);
    acc += classify16_0(1, acc, acc);
    acc += accum16_0(4);
    acc += guard16_0(acc);
    S16_1 s1 = mk16_1(acc);
    bump16_1(&s1, 6);
    acc += probe16_1(&s1);
    acc += read16_1(&s1);
    acc += classify16_1(1, acc, acc);
    acc += accum16_1(3);
    acc += guard16_1(acc);
    S16_2 s2 = mk16_2(acc);
    bump16_2(&s2, 8);
    acc += probe16_2(&s2);
    acc += read16_2(&s2);
    acc += classify16_2(1, acc, acc);
    acc += accum16_2(3);
    acc += guard16_2(acc);
    return clampi(acc);
}
