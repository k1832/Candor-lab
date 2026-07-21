/* GENERATED C mirror of reference module m157. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S157_0;

static S157_0 mk157_0(long a) {
    S157_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe157_0(const S157_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read157_0(const S157_0 *s) {
    return s->a * 6;
}
static void bump157_0(S157_0 *s, long d) {
    s->a = s->a + d;
}
static long classify157_0(int tag, long a, long b) {
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
static long accum157_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard157_0(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S157_1;

static S157_1 mk157_1(long a) {
    S157_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe157_1(const S157_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read157_1(const S157_1 *s) {
    return s->a * 5;
}
static void bump157_1(S157_1 *s, long d) {
    s->a = s->a + d;
}
static long classify157_1(int tag, long a, long b) {
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
static long accum157_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard157_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S157_2;

static S157_2 mk157_2(long a) {
    S157_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe157_2(const S157_2 *s) {
    return s->a + s->n0;
}
static long read157_2(const S157_2 *s) {
    return s->a * 7;
}
static void bump157_2(S157_2 *s, long d) {
    s->a = s->a + d;
}
static long classify157_2(int tag, long a, long b) {
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
static long accum157_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard157_2(long x) {
    return x + 4;
}

long f157(long x) {
    long acc = x;
    acc += f112(x + 1);
    S157_0 s0 = mk157_0(acc);
    bump157_0(&s0, 7);
    acc += probe157_0(&s0);
    acc += read157_0(&s0);
    acc += classify157_0(1, acc, acc);
    acc += accum157_0(7);
    acc += guard157_0(acc);
    S157_1 s1 = mk157_1(acc);
    bump157_1(&s1, 1);
    acc += probe157_1(&s1);
    acc += read157_1(&s1);
    acc += classify157_1(1, acc, acc);
    acc += accum157_1(3);
    acc += guard157_1(acc);
    S157_2 s2 = mk157_2(acc);
    bump157_2(&s2, 9);
    acc += probe157_2(&s2);
    acc += read157_2(&s2);
    acc += classify157_2(1, acc, acc);
    acc += accum157_2(3);
    acc += guard157_2(acc);
    return clampi(acc);
}
