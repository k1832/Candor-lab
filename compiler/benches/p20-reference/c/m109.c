/* GENERATED C mirror of reference module m109. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S109_0;

static S109_0 mk109_0(long a) {
    S109_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe109_0(const S109_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read109_0(const S109_0 *s) {
    return s->a * 6;
}
static void bump109_0(S109_0 *s, long d) {
    s->a = s->a + d;
}
static long classify109_0(int tag, long a, long b) {
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
static long accum109_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard109_0(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S109_1;

static S109_1 mk109_1(long a) {
    S109_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe109_1(const S109_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read109_1(const S109_1 *s) {
    return s->a * 7;
}
static void bump109_1(S109_1 *s, long d) {
    s->a = s->a + d;
}
static long classify109_1(int tag, long a, long b) {
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
static long accum109_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard109_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S109_2;

static S109_2 mk109_2(long a) {
    S109_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe109_2(const S109_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read109_2(const S109_2 *s) {
    return s->a * 6;
}
static void bump109_2(S109_2 *s, long d) {
    s->a = s->a + d;
}
static long classify109_2(int tag, long a, long b) {
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
static long accum109_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard109_2(long x) {
    return x + 2;
}

long f109(long x) {
    long acc = x;
    acc += f014(x + 1);
    S109_0 s0 = mk109_0(acc);
    bump109_0(&s0, 1);
    acc += probe109_0(&s0);
    acc += read109_0(&s0);
    acc += classify109_0(1, acc, acc);
    acc += accum109_0(4);
    acc += guard109_0(acc);
    S109_1 s1 = mk109_1(acc);
    bump109_1(&s1, 7);
    acc += probe109_1(&s1);
    acc += read109_1(&s1);
    acc += classify109_1(1, acc, acc);
    acc += accum109_1(4);
    acc += guard109_1(acc);
    S109_2 s2 = mk109_2(acc);
    bump109_2(&s2, 5);
    acc += probe109_2(&s2);
    acc += read109_2(&s2);
    acc += classify109_2(1, acc, acc);
    acc += accum109_2(6);
    acc += guard109_2(acc);
    return clampi(acc);
}
