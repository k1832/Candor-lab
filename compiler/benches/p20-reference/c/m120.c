/* GENERATED C mirror of reference module m120. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S120_0;

static S120_0 mk120_0(long a) {
    S120_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe120_0(const S120_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read120_0(const S120_0 *s) {
    return s->a * 4;
}
static void bump120_0(S120_0 *s, long d) {
    s->a = s->a + d;
}
static long classify120_0(int tag, long a, long b) {
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
static long accum120_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard120_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S120_1;

static S120_1 mk120_1(long a) {
    S120_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe120_1(const S120_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read120_1(const S120_1 *s) {
    return s->a * 5;
}
static void bump120_1(S120_1 *s, long d) {
    s->a = s->a + d;
}
static long classify120_1(int tag, long a, long b) {
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
static long accum120_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard120_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S120_2;

static S120_2 mk120_2(long a) {
    S120_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe120_2(const S120_2 *s) {
    return s->a + s->n0;
}
static long read120_2(const S120_2 *s) {
    return s->a * 6;
}
static void bump120_2(S120_2 *s, long d) {
    s->a = s->a + d;
}
static long classify120_2(int tag, long a, long b) {
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
static long accum120_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard120_2(long x) {
    return x + 4;
}

long f120(long x) {
    long acc = x;
    acc += f045(x + 1);
    acc += f059(x + 2);
    acc += f100(x + 3);
    acc += f101(x + 4);
    S120_0 s0 = mk120_0(acc);
    bump120_0(&s0, 8);
    acc += probe120_0(&s0);
    acc += read120_0(&s0);
    acc += classify120_0(1, acc, acc);
    acc += accum120_0(9);
    acc += guard120_0(acc);
    S120_1 s1 = mk120_1(acc);
    bump120_1(&s1, 5);
    acc += probe120_1(&s1);
    acc += read120_1(&s1);
    acc += classify120_1(1, acc, acc);
    acc += accum120_1(4);
    acc += guard120_1(acc);
    S120_2 s2 = mk120_2(acc);
    bump120_2(&s2, 7);
    acc += probe120_2(&s2);
    acc += read120_2(&s2);
    acc += classify120_2(1, acc, acc);
    acc += accum120_2(8);
    acc += guard120_2(acc);
    return clampi(acc);
}
