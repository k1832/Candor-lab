/* GENERATED C mirror of reference module m022. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S22_0;

static S22_0 mk22_0(long a) {
    S22_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe22_0(const S22_0 *s) {
    return s->a + s->n0;
}
static long read22_0(const S22_0 *s) {
    return s->a * 4;
}
static void bump22_0(S22_0 *s, long d) {
    s->a = s->a + d;
}
static long classify22_0(int tag, long a, long b) {
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
static long accum22_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard22_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S22_1;

static S22_1 mk22_1(long a) {
    S22_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe22_1(const S22_1 *s) {
    return s->a + s->n0;
}
static long read22_1(const S22_1 *s) {
    return s->a * 3;
}
static void bump22_1(S22_1 *s, long d) {
    s->a = s->a + d;
}
static long classify22_1(int tag, long a, long b) {
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
static long accum22_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard22_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S22_2;

static S22_2 mk22_2(long a) {
    S22_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe22_2(const S22_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read22_2(const S22_2 *s) {
    return s->a * 2;
}
static void bump22_2(S22_2 *s, long d) {
    s->a = s->a + d;
}
static long classify22_2(int tag, long a, long b) {
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
static long accum22_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard22_2(long x) {
    return x + 2;
}

long f022(long x) {
    long acc = x;
    acc += f000(x + 1);
    acc += f006(x + 2);
    S22_0 s0 = mk22_0(acc);
    bump22_0(&s0, 5);
    acc += probe22_0(&s0);
    acc += read22_0(&s0);
    acc += classify22_0(1, acc, acc);
    acc += accum22_0(6);
    acc += guard22_0(acc);
    S22_1 s1 = mk22_1(acc);
    bump22_1(&s1, 7);
    acc += probe22_1(&s1);
    acc += read22_1(&s1);
    acc += classify22_1(1, acc, acc);
    acc += accum22_1(9);
    acc += guard22_1(acc);
    S22_2 s2 = mk22_2(acc);
    bump22_2(&s2, 7);
    acc += probe22_2(&s2);
    acc += read22_2(&s2);
    acc += classify22_2(1, acc, acc);
    acc += accum22_2(6);
    acc += guard22_2(acc);
    return clampi(acc);
}
