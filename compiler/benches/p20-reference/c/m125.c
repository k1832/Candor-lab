/* GENERATED C mirror of reference module m125. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S125_0;

static S125_0 mk125_0(long a) {
    S125_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe125_0(const S125_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read125_0(const S125_0 *s) {
    return s->a * 6;
}
static void bump125_0(S125_0 *s, long d) {
    s->a = s->a + d;
}
static long classify125_0(int tag, long a, long b) {
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
static long accum125_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard125_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S125_1;

static S125_1 mk125_1(long a) {
    S125_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe125_1(const S125_1 *s) {
    return s->a + s->n0;
}
static long read125_1(const S125_1 *s) {
    return s->a * 6;
}
static void bump125_1(S125_1 *s, long d) {
    s->a = s->a + d;
}
static long classify125_1(int tag, long a, long b) {
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
static long accum125_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard125_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S125_2;

static S125_2 mk125_2(long a) {
    S125_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe125_2(const S125_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read125_2(const S125_2 *s) {
    return s->a * 5;
}
static void bump125_2(S125_2 *s, long d) {
    s->a = s->a + d;
}
static long classify125_2(int tag, long a, long b) {
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
static long accum125_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard125_2(long x) {
    return x + 7;
}

long f125(long x) {
    long acc = x;
    acc += f058(x + 1);
    acc += f064(x + 2);
    acc += f088(x + 3);
    S125_0 s0 = mk125_0(acc);
    bump125_0(&s0, 1);
    acc += probe125_0(&s0);
    acc += read125_0(&s0);
    acc += classify125_0(1, acc, acc);
    acc += accum125_0(3);
    acc += guard125_0(acc);
    S125_1 s1 = mk125_1(acc);
    bump125_1(&s1, 6);
    acc += probe125_1(&s1);
    acc += read125_1(&s1);
    acc += classify125_1(1, acc, acc);
    acc += accum125_1(4);
    acc += guard125_1(acc);
    S125_2 s2 = mk125_2(acc);
    bump125_2(&s2, 9);
    acc += probe125_2(&s2);
    acc += read125_2(&s2);
    acc += classify125_2(1, acc, acc);
    acc += accum125_2(3);
    acc += guard125_2(acc);
    return clampi(acc);
}
