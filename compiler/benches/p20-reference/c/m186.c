/* GENERATED C mirror of reference module m186. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S186_0;

static S186_0 mk186_0(long a) {
    S186_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe186_0(const S186_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read186_0(const S186_0 *s) {
    return s->a * 2;
}
static void bump186_0(S186_0 *s, long d) {
    s->a = s->a + d;
}
static long classify186_0(int tag, long a, long b) {
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
static long accum186_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard186_0(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S186_1;

static S186_1 mk186_1(long a) {
    S186_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe186_1(const S186_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read186_1(const S186_1 *s) {
    return s->a * 3;
}
static void bump186_1(S186_1 *s, long d) {
    s->a = s->a + d;
}
static long classify186_1(int tag, long a, long b) {
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
static long accum186_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard186_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S186_2;

static S186_2 mk186_2(long a) {
    S186_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe186_2(const S186_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read186_2(const S186_2 *s) {
    return s->a * 3;
}
static void bump186_2(S186_2 *s, long d) {
    s->a = s->a + d;
}
static long classify186_2(int tag, long a, long b) {
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
static long accum186_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard186_2(long x) {
    return x + 3;
}

long f186(long x) {
    long acc = x;
    acc += f026(x + 1);
    acc += f048(x + 2);
    acc += f068(x + 3);
    S186_0 s0 = mk186_0(acc);
    bump186_0(&s0, 5);
    acc += probe186_0(&s0);
    acc += read186_0(&s0);
    acc += classify186_0(1, acc, acc);
    acc += accum186_0(3);
    acc += guard186_0(acc);
    S186_1 s1 = mk186_1(acc);
    bump186_1(&s1, 7);
    acc += probe186_1(&s1);
    acc += read186_1(&s1);
    acc += classify186_1(1, acc, acc);
    acc += accum186_1(6);
    acc += guard186_1(acc);
    S186_2 s2 = mk186_2(acc);
    bump186_2(&s2, 9);
    acc += probe186_2(&s2);
    acc += read186_2(&s2);
    acc += classify186_2(1, acc, acc);
    acc += accum186_2(4);
    acc += guard186_2(acc);
    return clampi(acc);
}
