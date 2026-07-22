/* GENERATED C mirror of reference module m063. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S63_0;

static S63_0 mk63_0(long a) {
    S63_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe63_0(const S63_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read63_0(const S63_0 *s) {
    return s->a * 7;
}
static void bump63_0(S63_0 *s, long d) {
    s->a = s->a + d;
}
static long classify63_0(int tag, long a, long b) {
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
static long accum63_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard63_0(long x) {
    return x + 1;
}

static long pick63_0_0(long a, long b) { return a > b ? a : b; }
static long pick63_0_1(long a, long b) { return a > b ? a : b; }
static long pick63_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S63_1;

static S63_1 mk63_1(long a) {
    S63_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe63_1(const S63_1 *s) {
    return s->a + s->n0;
}
static long read63_1(const S63_1 *s) {
    return s->a * 6;
}
static void bump63_1(S63_1 *s, long d) {
    s->a = s->a + d;
}
static long classify63_1(int tag, long a, long b) {
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
static long accum63_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard63_1(long x) {
    return x + 1;
}

static long pick63_1_0(long a, long b) { return a > b ? a : b; }
static long pick63_1_1(long a, long b) { return a > b ? a : b; }
static long pick63_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S63_2;

static S63_2 mk63_2(long a) {
    S63_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe63_2(const S63_2 *s) {
    return s->a + s->n0;
}
static long read63_2(const S63_2 *s) {
    return s->a * 4;
}
static void bump63_2(S63_2 *s, long d) {
    s->a = s->a + d;
}
static long classify63_2(int tag, long a, long b) {
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
static long accum63_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard63_2(long x) {
    return x + 8;
}

static long pick63_2_0(long a, long b) { return a > b ? a : b; }
long f063(long x) {
    long acc = x;
    acc += f007(x + 1);
    S63_0 s0 = mk63_0(acc);
    bump63_0(&s0, 3);
    acc += probe63_0(&s0);
    acc += read63_0(&s0);
    acc += classify63_0(1, acc, acc);
    acc += accum63_0(4);
    acc += guard63_0(acc);
    acc += pick63_0_0(acc, acc + 6);
    acc += pick63_0_1(acc, acc + 8);
    acc += pick63_0_2(acc, acc + 3);
    S63_1 s1 = mk63_1(acc);
    bump63_1(&s1, 7);
    acc += probe63_1(&s1);
    acc += read63_1(&s1);
    acc += classify63_1(1, acc, acc);
    acc += accum63_1(5);
    acc += guard63_1(acc);
    acc += pick63_1_0(acc, acc + 7);
    acc += pick63_1_1(acc, acc + 5);
    acc += pick63_1_2(acc, acc + 8);
    S63_2 s2 = mk63_2(acc);
    bump63_2(&s2, 5);
    acc += probe63_2(&s2);
    acc += read63_2(&s2);
    acc += classify63_2(1, acc, acc);
    acc += accum63_2(5);
    acc += guard63_2(acc);
    acc += pick63_2_0(acc, acc + 9);
    return clampi(acc);
}
