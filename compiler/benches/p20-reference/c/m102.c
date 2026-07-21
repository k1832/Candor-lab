/* GENERATED C mirror of reference module m102. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S102_0;

static S102_0 mk102_0(long a) {
    S102_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe102_0(const S102_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read102_0(const S102_0 *s) {
    return s->a * 3;
}
static void bump102_0(S102_0 *s, long d) {
    s->a = s->a + d;
}
static long classify102_0(int tag, long a, long b) {
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
static long accum102_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard102_0(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S102_1;

static S102_1 mk102_1(long a) {
    S102_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe102_1(const S102_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read102_1(const S102_1 *s) {
    return s->a * 5;
}
static void bump102_1(S102_1 *s, long d) {
    s->a = s->a + d;
}
static long classify102_1(int tag, long a, long b) {
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
static long accum102_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard102_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S102_2;

static S102_2 mk102_2(long a) {
    S102_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe102_2(const S102_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read102_2(const S102_2 *s) {
    return s->a * 3;
}
static void bump102_2(S102_2 *s, long d) {
    s->a = s->a + d;
}
static long classify102_2(int tag, long a, long b) {
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
static long accum102_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard102_2(long x) {
    return x + 6;
}

long f102(long x) {
    long acc = x;
    acc += f057(x + 1);
    S102_0 s0 = mk102_0(acc);
    bump102_0(&s0, 9);
    acc += probe102_0(&s0);
    acc += read102_0(&s0);
    acc += classify102_0(1, acc, acc);
    acc += accum102_0(6);
    acc += guard102_0(acc);
    S102_1 s1 = mk102_1(acc);
    bump102_1(&s1, 6);
    acc += probe102_1(&s1);
    acc += read102_1(&s1);
    acc += classify102_1(1, acc, acc);
    acc += accum102_1(7);
    acc += guard102_1(acc);
    S102_2 s2 = mk102_2(acc);
    bump102_2(&s2, 4);
    acc += probe102_2(&s2);
    acc += read102_2(&s2);
    acc += classify102_2(1, acc, acc);
    acc += accum102_2(8);
    acc += guard102_2(acc);
    return clampi(acc);
}
