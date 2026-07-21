/* GENERATED C mirror of reference module m145. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S145_0;

static S145_0 mk145_0(long a) {
    S145_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe145_0(const S145_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read145_0(const S145_0 *s) {
    return s->a * 5;
}
static void bump145_0(S145_0 *s, long d) {
    s->a = s->a + d;
}
static long classify145_0(int tag, long a, long b) {
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
static long accum145_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard145_0(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S145_1;

static S145_1 mk145_1(long a) {
    S145_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe145_1(const S145_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read145_1(const S145_1 *s) {
    return s->a * 4;
}
static void bump145_1(S145_1 *s, long d) {
    s->a = s->a + d;
}
static long classify145_1(int tag, long a, long b) {
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
static long accum145_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard145_1(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S145_2;

static S145_2 mk145_2(long a) {
    S145_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe145_2(const S145_2 *s) {
    return s->a + s->n0;
}
static long read145_2(const S145_2 *s) {
    return s->a * 2;
}
static void bump145_2(S145_2 *s, long d) {
    s->a = s->a + d;
}
static long classify145_2(int tag, long a, long b) {
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
static long accum145_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard145_2(long x) {
    return x + 1;
}

long f145(long x) {
    long acc = x;
    acc += f066(x + 1);
    acc += f073(x + 2);
    acc += f139(x + 3);
    S145_0 s0 = mk145_0(acc);
    bump145_0(&s0, 2);
    acc += probe145_0(&s0);
    acc += read145_0(&s0);
    acc += classify145_0(1, acc, acc);
    acc += accum145_0(5);
    acc += guard145_0(acc);
    S145_1 s1 = mk145_1(acc);
    bump145_1(&s1, 5);
    acc += probe145_1(&s1);
    acc += read145_1(&s1);
    acc += classify145_1(1, acc, acc);
    acc += accum145_1(6);
    acc += guard145_1(acc);
    S145_2 s2 = mk145_2(acc);
    bump145_2(&s2, 8);
    acc += probe145_2(&s2);
    acc += read145_2(&s2);
    acc += classify145_2(1, acc, acc);
    acc += accum145_2(7);
    acc += guard145_2(acc);
    return clampi(acc);
}
