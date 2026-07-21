/* GENERATED C mirror of reference module m096. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S96_0;

static S96_0 mk96_0(long a) {
    S96_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe96_0(const S96_0 *s) {
    return s->a + s->n0;
}
static long read96_0(const S96_0 *s) {
    return s->a * 2;
}
static void bump96_0(S96_0 *s, long d) {
    s->a = s->a + d;
}
static long classify96_0(int tag, long a, long b) {
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
static long accum96_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard96_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S96_1;

static S96_1 mk96_1(long a) {
    S96_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe96_1(const S96_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read96_1(const S96_1 *s) {
    return s->a * 2;
}
static void bump96_1(S96_1 *s, long d) {
    s->a = s->a + d;
}
static long classify96_1(int tag, long a, long b) {
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
static long accum96_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard96_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S96_2;

static S96_2 mk96_2(long a) {
    S96_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe96_2(const S96_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read96_2(const S96_2 *s) {
    return s->a * 2;
}
static void bump96_2(S96_2 *s, long d) {
    s->a = s->a + d;
}
static long classify96_2(int tag, long a, long b) {
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
static long accum96_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard96_2(long x) {
    return x + 9;
}

long f096(long x) {
    long acc = x;
    acc += f018(x + 1);
    acc += f035(x + 2);
    acc += f068(x + 3);
    S96_0 s0 = mk96_0(acc);
    bump96_0(&s0, 6);
    acc += probe96_0(&s0);
    acc += read96_0(&s0);
    acc += classify96_0(1, acc, acc);
    acc += accum96_0(7);
    acc += guard96_0(acc);
    S96_1 s1 = mk96_1(acc);
    bump96_1(&s1, 3);
    acc += probe96_1(&s1);
    acc += read96_1(&s1);
    acc += classify96_1(1, acc, acc);
    acc += accum96_1(8);
    acc += guard96_1(acc);
    S96_2 s2 = mk96_2(acc);
    bump96_2(&s2, 7);
    acc += probe96_2(&s2);
    acc += read96_2(&s2);
    acc += classify96_2(1, acc, acc);
    acc += accum96_2(9);
    acc += guard96_2(acc);
    return clampi(acc);
}
