/* GENERATED C mirror of reference module m002. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S2_0;

static S2_0 mk2_0(long a) {
    S2_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe2_0(const S2_0 *s) {
    return s->a + s->n0;
}
static long read2_0(const S2_0 *s) {
    return s->a * 2;
}
static void bump2_0(S2_0 *s, long d) {
    s->a = s->a + d;
}
static long classify2_0(int tag, long a, long b) {
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
static long accum2_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard2_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S2_1;

static S2_1 mk2_1(long a) {
    S2_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe2_1(const S2_1 *s) {
    return s->a + s->n0;
}
static long read2_1(const S2_1 *s) {
    return s->a * 5;
}
static void bump2_1(S2_1 *s, long d) {
    s->a = s->a + d;
}
static long classify2_1(int tag, long a, long b) {
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
static long accum2_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard2_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S2_2;

static S2_2 mk2_2(long a) {
    S2_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe2_2(const S2_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read2_2(const S2_2 *s) {
    return s->a * 6;
}
static void bump2_2(S2_2 *s, long d) {
    s->a = s->a + d;
}
static long classify2_2(int tag, long a, long b) {
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
static long accum2_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard2_2(long x) {
    return x + 4;
}

long f002(long x) {
    long acc = x;
    S2_0 s0 = mk2_0(acc);
    bump2_0(&s0, 4);
    acc += probe2_0(&s0);
    acc += read2_0(&s0);
    acc += classify2_0(1, acc, acc);
    acc += accum2_0(7);
    acc += guard2_0(acc);
    S2_1 s1 = mk2_1(acc);
    bump2_1(&s1, 7);
    acc += probe2_1(&s1);
    acc += read2_1(&s1);
    acc += classify2_1(1, acc, acc);
    acc += accum2_1(6);
    acc += guard2_1(acc);
    S2_2 s2 = mk2_2(acc);
    bump2_2(&s2, 9);
    acc += probe2_2(&s2);
    acc += read2_2(&s2);
    acc += classify2_2(1, acc, acc);
    acc += accum2_2(8);
    acc += guard2_2(acc);
    return clampi(acc);
}
