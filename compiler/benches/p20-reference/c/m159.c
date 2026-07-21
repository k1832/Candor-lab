/* GENERATED C mirror of reference module m159. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S159_0;

static S159_0 mk159_0(long a) {
    S159_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe159_0(const S159_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read159_0(const S159_0 *s) {
    return s->a * 6;
}
static void bump159_0(S159_0 *s, long d) {
    s->a = s->a + d;
}
static long classify159_0(int tag, long a, long b) {
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
static long accum159_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard159_0(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S159_1;

static S159_1 mk159_1(long a) {
    S159_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe159_1(const S159_1 *s) {
    return s->a + s->n0;
}
static long read159_1(const S159_1 *s) {
    return s->a * 3;
}
static void bump159_1(S159_1 *s, long d) {
    s->a = s->a + d;
}
static long classify159_1(int tag, long a, long b) {
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
static long accum159_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard159_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S159_2;

static S159_2 mk159_2(long a) {
    S159_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe159_2(const S159_2 *s) {
    return s->a + s->n0;
}
static long read159_2(const S159_2 *s) {
    return s->a * 3;
}
static void bump159_2(S159_2 *s, long d) {
    s->a = s->a + d;
}
static long classify159_2(int tag, long a, long b) {
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
static long accum159_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard159_2(long x) {
    return x + 2;
}

long f159(long x) {
    long acc = x;
    acc += f000(x + 1);
    S159_0 s0 = mk159_0(acc);
    bump159_0(&s0, 9);
    acc += probe159_0(&s0);
    acc += read159_0(&s0);
    acc += classify159_0(1, acc, acc);
    acc += accum159_0(9);
    acc += guard159_0(acc);
    S159_1 s1 = mk159_1(acc);
    bump159_1(&s1, 6);
    acc += probe159_1(&s1);
    acc += read159_1(&s1);
    acc += classify159_1(1, acc, acc);
    acc += accum159_1(7);
    acc += guard159_1(acc);
    S159_2 s2 = mk159_2(acc);
    bump159_2(&s2, 4);
    acc += probe159_2(&s2);
    acc += read159_2(&s2);
    acc += classify159_2(1, acc, acc);
    acc += accum159_2(8);
    acc += guard159_2(acc);
    return clampi(acc);
}
