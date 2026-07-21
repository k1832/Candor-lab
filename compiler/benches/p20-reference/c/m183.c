/* GENERATED C mirror of reference module m183. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S183_0;

static S183_0 mk183_0(long a) {
    S183_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe183_0(const S183_0 *s) {
    return s->a + s->n0;
}
static long read183_0(const S183_0 *s) {
    return s->a * 5;
}
static void bump183_0(S183_0 *s, long d) {
    s->a = s->a + d;
}
static long classify183_0(int tag, long a, long b) {
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
static long accum183_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard183_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S183_1;

static S183_1 mk183_1(long a) {
    S183_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe183_1(const S183_1 *s) {
    return s->a + s->n0;
}
static long read183_1(const S183_1 *s) {
    return s->a * 7;
}
static void bump183_1(S183_1 *s, long d) {
    s->a = s->a + d;
}
static long classify183_1(int tag, long a, long b) {
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
static long accum183_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard183_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S183_2;

static S183_2 mk183_2(long a) {
    S183_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe183_2(const S183_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read183_2(const S183_2 *s) {
    return s->a * 5;
}
static void bump183_2(S183_2 *s, long d) {
    s->a = s->a + d;
}
static long classify183_2(int tag, long a, long b) {
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
static long accum183_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard183_2(long x) {
    return x + 1;
}

long f183(long x) {
    long acc = x;
    acc += f068(x + 1);
    acc += f080(x + 2);
    S183_0 s0 = mk183_0(acc);
    bump183_0(&s0, 3);
    acc += probe183_0(&s0);
    acc += read183_0(&s0);
    acc += classify183_0(1, acc, acc);
    acc += accum183_0(3);
    acc += guard183_0(acc);
    S183_1 s1 = mk183_1(acc);
    bump183_1(&s1, 6);
    acc += probe183_1(&s1);
    acc += read183_1(&s1);
    acc += classify183_1(1, acc, acc);
    acc += accum183_1(6);
    acc += guard183_1(acc);
    S183_2 s2 = mk183_2(acc);
    bump183_2(&s2, 7);
    acc += probe183_2(&s2);
    acc += read183_2(&s2);
    acc += classify183_2(1, acc, acc);
    acc += accum183_2(5);
    acc += guard183_2(acc);
    return clampi(acc);
}
