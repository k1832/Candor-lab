/* GENERATED C mirror of reference module m000. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S0_0;

static S0_0 mk0_0(long a) {
    S0_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe0_0(const S0_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read0_0(const S0_0 *s) {
    return s->a * 3;
}
static void bump0_0(S0_0 *s, long d) {
    s->a = s->a + d;
}
static long classify0_0(int tag, long a, long b) {
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
static long accum0_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard0_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S0_1;

static S0_1 mk0_1(long a) {
    S0_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe0_1(const S0_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read0_1(const S0_1 *s) {
    return s->a * 6;
}
static void bump0_1(S0_1 *s, long d) {
    s->a = s->a + d;
}
static long classify0_1(int tag, long a, long b) {
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
static long accum0_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard0_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S0_2;

static S0_2 mk0_2(long a) {
    S0_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe0_2(const S0_2 *s) {
    return s->a + s->n0;
}
static long read0_2(const S0_2 *s) {
    return s->a * 4;
}
static void bump0_2(S0_2 *s, long d) {
    s->a = s->a + d;
}
static long classify0_2(int tag, long a, long b) {
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
static long accum0_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard0_2(long x) {
    return x + 7;
}

long f000(long x) {
    long acc = x;
    S0_0 s0 = mk0_0(acc);
    bump0_0(&s0, 6);
    acc += probe0_0(&s0);
    acc += read0_0(&s0);
    acc += classify0_0(1, acc, acc);
    acc += accum0_0(7);
    acc += guard0_0(acc);
    S0_1 s1 = mk0_1(acc);
    bump0_1(&s1, 9);
    acc += probe0_1(&s1);
    acc += read0_1(&s1);
    acc += classify0_1(1, acc, acc);
    acc += accum0_1(6);
    acc += guard0_1(acc);
    S0_2 s2 = mk0_2(acc);
    bump0_2(&s2, 4);
    acc += probe0_2(&s2);
    acc += read0_2(&s2);
    acc += classify0_2(1, acc, acc);
    acc += accum0_2(8);
    acc += guard0_2(acc);
    return clampi(acc);
}
