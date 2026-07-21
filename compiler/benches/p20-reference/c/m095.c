/* GENERATED C mirror of reference module m095. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S95_0;

static S95_0 mk95_0(long a) {
    S95_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe95_0(const S95_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read95_0(const S95_0 *s) {
    return s->a * 3;
}
static void bump95_0(S95_0 *s, long d) {
    s->a = s->a + d;
}
static long classify95_0(int tag, long a, long b) {
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
static long accum95_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard95_0(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S95_1;

static S95_1 mk95_1(long a) {
    S95_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe95_1(const S95_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read95_1(const S95_1 *s) {
    return s->a * 7;
}
static void bump95_1(S95_1 *s, long d) {
    s->a = s->a + d;
}
static long classify95_1(int tag, long a, long b) {
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
static long accum95_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard95_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S95_2;

static S95_2 mk95_2(long a) {
    S95_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe95_2(const S95_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read95_2(const S95_2 *s) {
    return s->a * 3;
}
static void bump95_2(S95_2 *s, long d) {
    s->a = s->a + d;
}
static long classify95_2(int tag, long a, long b) {
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
static long accum95_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard95_2(long x) {
    return x + 4;
}

long f095(long x) {
    long acc = x;
    acc += f040(x + 1);
    acc += f061(x + 2);
    acc += f066(x + 3);
    S95_0 s0 = mk95_0(acc);
    bump95_0(&s0, 9);
    acc += probe95_0(&s0);
    acc += read95_0(&s0);
    acc += classify95_0(1, acc, acc);
    acc += accum95_0(8);
    acc += guard95_0(acc);
    S95_1 s1 = mk95_1(acc);
    bump95_1(&s1, 6);
    acc += probe95_1(&s1);
    acc += read95_1(&s1);
    acc += classify95_1(1, acc, acc);
    acc += accum95_1(3);
    acc += guard95_1(acc);
    S95_2 s2 = mk95_2(acc);
    bump95_2(&s2, 1);
    acc += probe95_2(&s2);
    acc += read95_2(&s2);
    acc += classify95_2(1, acc, acc);
    acc += accum95_2(5);
    acc += guard95_2(acc);
    return clampi(acc);
}
