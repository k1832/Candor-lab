/* GENERATED C mirror of reference module m164. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S164_0;

static S164_0 mk164_0(long a) {
    S164_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe164_0(const S164_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read164_0(const S164_0 *s) {
    return s->a * 7;
}
static void bump164_0(S164_0 *s, long d) {
    s->a = s->a + d;
}
static long classify164_0(int tag, long a, long b) {
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
static long accum164_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard164_0(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S164_1;

static S164_1 mk164_1(long a) {
    S164_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe164_1(const S164_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read164_1(const S164_1 *s) {
    return s->a * 2;
}
static void bump164_1(S164_1 *s, long d) {
    s->a = s->a + d;
}
static long classify164_1(int tag, long a, long b) {
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
static long accum164_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard164_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S164_2;

static S164_2 mk164_2(long a) {
    S164_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe164_2(const S164_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read164_2(const S164_2 *s) {
    return s->a * 4;
}
static void bump164_2(S164_2 *s, long d) {
    s->a = s->a + d;
}
static long classify164_2(int tag, long a, long b) {
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
static long accum164_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard164_2(long x) {
    return x + 3;
}

long f164(long x) {
    long acc = x;
    acc += f102(x + 1);
    acc += f110(x + 2);
    acc += f137(x + 3);
    S164_0 s0 = mk164_0(acc);
    bump164_0(&s0, 7);
    acc += probe164_0(&s0);
    acc += read164_0(&s0);
    acc += classify164_0(1, acc, acc);
    acc += accum164_0(6);
    acc += guard164_0(acc);
    S164_1 s1 = mk164_1(acc);
    bump164_1(&s1, 9);
    acc += probe164_1(&s1);
    acc += read164_1(&s1);
    acc += classify164_1(1, acc, acc);
    acc += accum164_1(6);
    acc += guard164_1(acc);
    S164_2 s2 = mk164_2(acc);
    bump164_2(&s2, 7);
    acc += probe164_2(&s2);
    acc += read164_2(&s2);
    acc += classify164_2(1, acc, acc);
    acc += accum164_2(6);
    acc += guard164_2(acc);
    return clampi(acc);
}
