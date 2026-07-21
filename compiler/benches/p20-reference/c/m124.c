/* GENERATED C mirror of reference module m124. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S124_0;

static S124_0 mk124_0(long a) {
    S124_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe124_0(const S124_0 *s) {
    return s->a + s->n0;
}
static long read124_0(const S124_0 *s) {
    return s->a * 3;
}
static void bump124_0(S124_0 *s, long d) {
    s->a = s->a + d;
}
static long classify124_0(int tag, long a, long b) {
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
static long accum124_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard124_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S124_1;

static S124_1 mk124_1(long a) {
    S124_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe124_1(const S124_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read124_1(const S124_1 *s) {
    return s->a * 3;
}
static void bump124_1(S124_1 *s, long d) {
    s->a = s->a + d;
}
static long classify124_1(int tag, long a, long b) {
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
static long accum124_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard124_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S124_2;

static S124_2 mk124_2(long a) {
    S124_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe124_2(const S124_2 *s) {
    return s->a + s->n0;
}
static long read124_2(const S124_2 *s) {
    return s->a * 2;
}
static void bump124_2(S124_2 *s, long d) {
    s->a = s->a + d;
}
static long classify124_2(int tag, long a, long b) {
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
static long accum124_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard124_2(long x) {
    return x + 8;
}

long f124(long x) {
    long acc = x;
    acc += f055(x + 1);
    acc += f086(x + 2);
    S124_0 s0 = mk124_0(acc);
    bump124_0(&s0, 4);
    acc += probe124_0(&s0);
    acc += read124_0(&s0);
    acc += classify124_0(1, acc, acc);
    acc += accum124_0(4);
    acc += guard124_0(acc);
    S124_1 s1 = mk124_1(acc);
    bump124_1(&s1, 1);
    acc += probe124_1(&s1);
    acc += read124_1(&s1);
    acc += classify124_1(1, acc, acc);
    acc += accum124_1(3);
    acc += guard124_1(acc);
    S124_2 s2 = mk124_2(acc);
    bump124_2(&s2, 9);
    acc += probe124_2(&s2);
    acc += read124_2(&s2);
    acc += classify124_2(1, acc, acc);
    acc += accum124_2(9);
    acc += guard124_2(acc);
    return clampi(acc);
}
