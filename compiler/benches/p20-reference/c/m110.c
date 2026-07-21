/* GENERATED C mirror of reference module m110. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S110_0;

static S110_0 mk110_0(long a) {
    S110_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe110_0(const S110_0 *s) {
    return s->a + s->n0;
}
static long read110_0(const S110_0 *s) {
    return s->a * 6;
}
static void bump110_0(S110_0 *s, long d) {
    s->a = s->a + d;
}
static long classify110_0(int tag, long a, long b) {
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
static long accum110_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard110_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S110_1;

static S110_1 mk110_1(long a) {
    S110_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe110_1(const S110_1 *s) {
    return s->a + s->n0;
}
static long read110_1(const S110_1 *s) {
    return s->a * 5;
}
static void bump110_1(S110_1 *s, long d) {
    s->a = s->a + d;
}
static long classify110_1(int tag, long a, long b) {
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
static long accum110_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard110_1(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S110_2;

static S110_2 mk110_2(long a) {
    S110_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe110_2(const S110_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read110_2(const S110_2 *s) {
    return s->a * 3;
}
static void bump110_2(S110_2 *s, long d) {
    s->a = s->a + d;
}
static long classify110_2(int tag, long a, long b) {
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
static long accum110_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard110_2(long x) {
    return x + 8;
}

long f110(long x) {
    long acc = x;
    acc += f018(x + 1);
    acc += f020(x + 2);
    acc += f054(x + 3);
    acc += f055(x + 4);
    S110_0 s0 = mk110_0(acc);
    bump110_0(&s0, 9);
    acc += probe110_0(&s0);
    acc += read110_0(&s0);
    acc += classify110_0(1, acc, acc);
    acc += accum110_0(8);
    acc += guard110_0(acc);
    S110_1 s1 = mk110_1(acc);
    bump110_1(&s1, 5);
    acc += probe110_1(&s1);
    acc += read110_1(&s1);
    acc += classify110_1(1, acc, acc);
    acc += accum110_1(4);
    acc += guard110_1(acc);
    S110_2 s2 = mk110_2(acc);
    bump110_2(&s2, 7);
    acc += probe110_2(&s2);
    acc += read110_2(&s2);
    acc += classify110_2(1, acc, acc);
    acc += accum110_2(8);
    acc += guard110_2(acc);
    return clampi(acc);
}
