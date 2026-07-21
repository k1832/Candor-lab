/* GENERATED C mirror of reference module m050. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S50_0;

static S50_0 mk50_0(long a) {
    S50_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe50_0(const S50_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read50_0(const S50_0 *s) {
    return s->a * 4;
}
static void bump50_0(S50_0 *s, long d) {
    s->a = s->a + d;
}
static long classify50_0(int tag, long a, long b) {
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
static long accum50_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard50_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S50_1;

static S50_1 mk50_1(long a) {
    S50_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe50_1(const S50_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read50_1(const S50_1 *s) {
    return s->a * 3;
}
static void bump50_1(S50_1 *s, long d) {
    s->a = s->a + d;
}
static long classify50_1(int tag, long a, long b) {
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
static long accum50_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard50_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S50_2;

static S50_2 mk50_2(long a) {
    S50_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe50_2(const S50_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read50_2(const S50_2 *s) {
    return s->a * 5;
}
static void bump50_2(S50_2 *s, long d) {
    s->a = s->a + d;
}
static long classify50_2(int tag, long a, long b) {
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
static long accum50_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard50_2(long x) {
    return x + 8;
}

long f050(long x) {
    long acc = x;
    acc += f013(x + 1);
    acc += f023(x + 2);
    S50_0 s0 = mk50_0(acc);
    bump50_0(&s0, 9);
    acc += probe50_0(&s0);
    acc += read50_0(&s0);
    acc += classify50_0(1, acc, acc);
    acc += accum50_0(7);
    acc += guard50_0(acc);
    S50_1 s1 = mk50_1(acc);
    bump50_1(&s1, 6);
    acc += probe50_1(&s1);
    acc += read50_1(&s1);
    acc += classify50_1(1, acc, acc);
    acc += accum50_1(5);
    acc += guard50_1(acc);
    S50_2 s2 = mk50_2(acc);
    bump50_2(&s2, 6);
    acc += probe50_2(&s2);
    acc += read50_2(&s2);
    acc += classify50_2(1, acc, acc);
    acc += accum50_2(7);
    acc += guard50_2(acc);
    return clampi(acc);
}
