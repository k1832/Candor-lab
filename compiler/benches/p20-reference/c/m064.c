/* GENERATED C mirror of reference module m064. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S64_0;

static S64_0 mk64_0(long a) {
    S64_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe64_0(const S64_0 *s) {
    return s->a + s->n0;
}
static long read64_0(const S64_0 *s) {
    return s->a * 2;
}
static void bump64_0(S64_0 *s, long d) {
    s->a = s->a + d;
}
static long classify64_0(int tag, long a, long b) {
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
static long accum64_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard64_0(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S64_1;

static S64_1 mk64_1(long a) {
    S64_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe64_1(const S64_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read64_1(const S64_1 *s) {
    return s->a * 4;
}
static void bump64_1(S64_1 *s, long d) {
    s->a = s->a + d;
}
static long classify64_1(int tag, long a, long b) {
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
static long accum64_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard64_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S64_2;

static S64_2 mk64_2(long a) {
    S64_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe64_2(const S64_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read64_2(const S64_2 *s) {
    return s->a * 7;
}
static void bump64_2(S64_2 *s, long d) {
    s->a = s->a + d;
}
static long classify64_2(int tag, long a, long b) {
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
static long accum64_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard64_2(long x) {
    return x + 2;
}

long f064(long x) {
    long acc = x;
    acc += f002(x + 1);
    acc += f023(x + 2);
    acc += f026(x + 3);
    acc += f035(x + 4);
    S64_0 s0 = mk64_0(acc);
    bump64_0(&s0, 2);
    acc += probe64_0(&s0);
    acc += read64_0(&s0);
    acc += classify64_0(1, acc, acc);
    acc += accum64_0(4);
    acc += guard64_0(acc);
    S64_1 s1 = mk64_1(acc);
    bump64_1(&s1, 1);
    acc += probe64_1(&s1);
    acc += read64_1(&s1);
    acc += classify64_1(1, acc, acc);
    acc += accum64_1(8);
    acc += guard64_1(acc);
    S64_2 s2 = mk64_2(acc);
    bump64_2(&s2, 9);
    acc += probe64_2(&s2);
    acc += read64_2(&s2);
    acc += classify64_2(1, acc, acc);
    acc += accum64_2(4);
    acc += guard64_2(acc);
    return clampi(acc);
}
