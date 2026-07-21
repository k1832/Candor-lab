/* GENERATED C mirror of reference module m075. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S75_0;

static S75_0 mk75_0(long a) {
    S75_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe75_0(const S75_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read75_0(const S75_0 *s) {
    return s->a * 4;
}
static void bump75_0(S75_0 *s, long d) {
    s->a = s->a + d;
}
static long classify75_0(int tag, long a, long b) {
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
static long accum75_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard75_0(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S75_1;

static S75_1 mk75_1(long a) {
    S75_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe75_1(const S75_1 *s) {
    return s->a + s->n0;
}
static long read75_1(const S75_1 *s) {
    return s->a * 3;
}
static void bump75_1(S75_1 *s, long d) {
    s->a = s->a + d;
}
static long classify75_1(int tag, long a, long b) {
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
static long accum75_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard75_1(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S75_2;

static S75_2 mk75_2(long a) {
    S75_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe75_2(const S75_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read75_2(const S75_2 *s) {
    return s->a * 7;
}
static void bump75_2(S75_2 *s, long d) {
    s->a = s->a + d;
}
static long classify75_2(int tag, long a, long b) {
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
static long accum75_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard75_2(long x) {
    return x + 6;
}

long f075(long x) {
    long acc = x;
    acc += f029(x + 1);
    acc += f046(x + 2);
    S75_0 s0 = mk75_0(acc);
    bump75_0(&s0, 4);
    acc += probe75_0(&s0);
    acc += read75_0(&s0);
    acc += classify75_0(1, acc, acc);
    acc += accum75_0(3);
    acc += guard75_0(acc);
    S75_1 s1 = mk75_1(acc);
    bump75_1(&s1, 8);
    acc += probe75_1(&s1);
    acc += read75_1(&s1);
    acc += classify75_1(1, acc, acc);
    acc += accum75_1(3);
    acc += guard75_1(acc);
    S75_2 s2 = mk75_2(acc);
    bump75_2(&s2, 3);
    acc += probe75_2(&s2);
    acc += read75_2(&s2);
    acc += classify75_2(1, acc, acc);
    acc += accum75_2(9);
    acc += guard75_2(acc);
    return clampi(acc);
}
