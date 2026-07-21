/* GENERATED C mirror of reference module m058. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S58_0;

static S58_0 mk58_0(long a) {
    S58_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe58_0(const S58_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read58_0(const S58_0 *s) {
    return s->a * 2;
}
static void bump58_0(S58_0 *s, long d) {
    s->a = s->a + d;
}
static long classify58_0(int tag, long a, long b) {
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
static long accum58_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard58_0(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S58_1;

static S58_1 mk58_1(long a) {
    S58_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe58_1(const S58_1 *s) {
    return s->a + s->n0;
}
static long read58_1(const S58_1 *s) {
    return s->a * 6;
}
static void bump58_1(S58_1 *s, long d) {
    s->a = s->a + d;
}
static long classify58_1(int tag, long a, long b) {
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
static long accum58_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard58_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S58_2;

static S58_2 mk58_2(long a) {
    S58_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe58_2(const S58_2 *s) {
    return s->a + s->n0;
}
static long read58_2(const S58_2 *s) {
    return s->a * 5;
}
static void bump58_2(S58_2 *s, long d) {
    s->a = s->a + d;
}
static long classify58_2(int tag, long a, long b) {
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
static long accum58_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard58_2(long x) {
    return x + 7;
}

long f058(long x) {
    long acc = x;
    acc += f030(x + 1);
    S58_0 s0 = mk58_0(acc);
    bump58_0(&s0, 9);
    acc += probe58_0(&s0);
    acc += read58_0(&s0);
    acc += classify58_0(1, acc, acc);
    acc += accum58_0(4);
    acc += guard58_0(acc);
    S58_1 s1 = mk58_1(acc);
    bump58_1(&s1, 2);
    acc += probe58_1(&s1);
    acc += read58_1(&s1);
    acc += classify58_1(1, acc, acc);
    acc += accum58_1(6);
    acc += guard58_1(acc);
    S58_2 s2 = mk58_2(acc);
    bump58_2(&s2, 3);
    acc += probe58_2(&s2);
    acc += read58_2(&s2);
    acc += classify58_2(1, acc, acc);
    acc += accum58_2(4);
    acc += guard58_2(acc);
    return clampi(acc);
}
