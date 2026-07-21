/* GENERATED C mirror of reference module m108. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S108_0;

static S108_0 mk108_0(long a) {
    S108_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe108_0(const S108_0 *s) {
    return s->a + s->n0;
}
static long read108_0(const S108_0 *s) {
    return s->a * 4;
}
static void bump108_0(S108_0 *s, long d) {
    s->a = s->a + d;
}
static long classify108_0(int tag, long a, long b) {
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
static long accum108_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard108_0(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S108_1;

static S108_1 mk108_1(long a) {
    S108_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe108_1(const S108_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read108_1(const S108_1 *s) {
    return s->a * 5;
}
static void bump108_1(S108_1 *s, long d) {
    s->a = s->a + d;
}
static long classify108_1(int tag, long a, long b) {
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
static long accum108_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard108_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S108_2;

static S108_2 mk108_2(long a) {
    S108_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe108_2(const S108_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read108_2(const S108_2 *s) {
    return s->a * 7;
}
static void bump108_2(S108_2 *s, long d) {
    s->a = s->a + d;
}
static long classify108_2(int tag, long a, long b) {
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
static long accum108_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard108_2(long x) {
    return x + 7;
}

long f108(long x) {
    long acc = x;
    acc += f043(x + 1);
    S108_0 s0 = mk108_0(acc);
    bump108_0(&s0, 6);
    acc += probe108_0(&s0);
    acc += read108_0(&s0);
    acc += classify108_0(1, acc, acc);
    acc += accum108_0(8);
    acc += guard108_0(acc);
    S108_1 s1 = mk108_1(acc);
    bump108_1(&s1, 9);
    acc += probe108_1(&s1);
    acc += read108_1(&s1);
    acc += classify108_1(1, acc, acc);
    acc += accum108_1(6);
    acc += guard108_1(acc);
    S108_2 s2 = mk108_2(acc);
    bump108_2(&s2, 5);
    acc += probe108_2(&s2);
    acc += read108_2(&s2);
    acc += classify108_2(1, acc, acc);
    acc += accum108_2(8);
    acc += guard108_2(acc);
    return clampi(acc);
}
