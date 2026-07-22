/* GENERATED C mirror of reference module m101. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S101_0;

static S101_0 mk101_0(long a) {
    S101_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe101_0(const S101_0 *s) {
    return s->a + s->n0;
}
static long read101_0(const S101_0 *s) {
    return s->a * 3;
}
static void bump101_0(S101_0 *s, long d) {
    s->a = s->a + d;
}
static long classify101_0(int tag, long a, long b) {
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
static long accum101_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard101_0(long x) {
    return x + 3;
}

static long pick101_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S101_1;

static S101_1 mk101_1(long a) {
    S101_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe101_1(const S101_1 *s) {
    return s->a + s->n0;
}
static long read101_1(const S101_1 *s) {
    return s->a * 6;
}
static void bump101_1(S101_1 *s, long d) {
    s->a = s->a + d;
}
static long classify101_1(int tag, long a, long b) {
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
static long accum101_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard101_1(long x) {
    return x + 7;
}

static long pick101_1_0(long a, long b) { return a > b ? a : b; }
static long pick101_1_1(long a, long b) { return a > b ? a : b; }
static long pick101_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S101_2;

static S101_2 mk101_2(long a) {
    S101_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe101_2(const S101_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read101_2(const S101_2 *s) {
    return s->a * 2;
}
static void bump101_2(S101_2 *s, long d) {
    s->a = s->a + d;
}
static long classify101_2(int tag, long a, long b) {
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
static long accum101_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard101_2(long x) {
    return x + 6;
}

static long pick101_2_0(long a, long b) { return a > b ? a : b; }
long f101(long x) {
    long acc = x;
    acc += f019(x + 1);
    acc += f077(x + 2);
    S101_0 s0 = mk101_0(acc);
    bump101_0(&s0, 4);
    acc += probe101_0(&s0);
    acc += read101_0(&s0);
    acc += classify101_0(1, acc, acc);
    acc += accum101_0(9);
    acc += guard101_0(acc);
    acc += pick101_0_0(acc, acc + 5);
    S101_1 s1 = mk101_1(acc);
    bump101_1(&s1, 8);
    acc += probe101_1(&s1);
    acc += read101_1(&s1);
    acc += classify101_1(1, acc, acc);
    acc += accum101_1(8);
    acc += guard101_1(acc);
    acc += pick101_1_0(acc, acc + 9);
    acc += pick101_1_1(acc, acc + 5);
    acc += pick101_1_2(acc, acc + 6);
    S101_2 s2 = mk101_2(acc);
    bump101_2(&s2, 8);
    acc += probe101_2(&s2);
    acc += read101_2(&s2);
    acc += classify101_2(1, acc, acc);
    acc += accum101_2(7);
    acc += guard101_2(acc);
    acc += pick101_2_0(acc, acc + 8);
    return clampi(acc);
}
