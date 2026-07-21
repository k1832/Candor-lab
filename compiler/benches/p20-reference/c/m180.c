/* GENERATED C mirror of reference module m180. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S180_0;

static S180_0 mk180_0(long a) {
    S180_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe180_0(const S180_0 *s) {
    return s->a + s->n0;
}
static long read180_0(const S180_0 *s) {
    return s->a * 4;
}
static void bump180_0(S180_0 *s, long d) {
    s->a = s->a + d;
}
static long classify180_0(int tag, long a, long b) {
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
static long accum180_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard180_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S180_1;

static S180_1 mk180_1(long a) {
    S180_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe180_1(const S180_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read180_1(const S180_1 *s) {
    return s->a * 4;
}
static void bump180_1(S180_1 *s, long d) {
    s->a = s->a + d;
}
static long classify180_1(int tag, long a, long b) {
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
static long accum180_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard180_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S180_2;

static S180_2 mk180_2(long a) {
    S180_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe180_2(const S180_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read180_2(const S180_2 *s) {
    return s->a * 5;
}
static void bump180_2(S180_2 *s, long d) {
    s->a = s->a + d;
}
static long classify180_2(int tag, long a, long b) {
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
static long accum180_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard180_2(long x) {
    return x + 5;
}

long f180(long x) {
    long acc = x;
    acc += f028(x + 1);
    acc += f060(x + 2);
    acc += f108(x + 3);
    S180_0 s0 = mk180_0(acc);
    bump180_0(&s0, 7);
    acc += probe180_0(&s0);
    acc += read180_0(&s0);
    acc += classify180_0(1, acc, acc);
    acc += accum180_0(4);
    acc += guard180_0(acc);
    S180_1 s1 = mk180_1(acc);
    bump180_1(&s1, 9);
    acc += probe180_1(&s1);
    acc += read180_1(&s1);
    acc += classify180_1(1, acc, acc);
    acc += accum180_1(8);
    acc += guard180_1(acc);
    S180_2 s2 = mk180_2(acc);
    bump180_2(&s2, 3);
    acc += probe180_2(&s2);
    acc += read180_2(&s2);
    acc += classify180_2(1, acc, acc);
    acc += accum180_2(3);
    acc += guard180_2(acc);
    return clampi(acc);
}
