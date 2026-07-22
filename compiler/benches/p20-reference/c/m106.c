/* GENERATED C mirror of reference module m106. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S106_0;

static S106_0 mk106_0(long a) {
    S106_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe106_0(const S106_0 *s) {
    return s->a + s->n0;
}
static long read106_0(const S106_0 *s) {
    return s->a * 2;
}
static void bump106_0(S106_0 *s, long d) {
    s->a = s->a + d;
}
static long classify106_0(int tag, long a, long b) {
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
static long accum106_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard106_0(long x) {
    return x + 9;
}

static long pick106_0_0(long a, long b) { return a > b ? a : b; }
static long pick106_0_1(long a, long b) { return a > b ? a : b; }
static long pick106_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S106_1;

static S106_1 mk106_1(long a) {
    S106_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe106_1(const S106_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read106_1(const S106_1 *s) {
    return s->a * 4;
}
static void bump106_1(S106_1 *s, long d) {
    s->a = s->a + d;
}
static long classify106_1(int tag, long a, long b) {
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
static long accum106_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard106_1(long x) {
    return x + 3;
}

static long pick106_1_0(long a, long b) { return a > b ? a : b; }
static long pick106_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S106_2;

static S106_2 mk106_2(long a) {
    S106_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe106_2(const S106_2 *s) {
    return s->a + s->n0;
}
static long read106_2(const S106_2 *s) {
    return s->a * 3;
}
static void bump106_2(S106_2 *s, long d) {
    s->a = s->a + d;
}
static long classify106_2(int tag, long a, long b) {
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
static long accum106_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard106_2(long x) {
    return x + 9;
}

static long pick106_2_0(long a, long b) { return a > b ? a : b; }
static long pick106_2_1(long a, long b) { return a > b ? a : b; }
static long pick106_2_2(long a, long b) { return a > b ? a : b; }
long f106(long x) {
    long acc = x;
    acc += f035(x + 1);
    acc += f056(x + 2);
    S106_0 s0 = mk106_0(acc);
    bump106_0(&s0, 4);
    acc += probe106_0(&s0);
    acc += read106_0(&s0);
    acc += classify106_0(1, acc, acc);
    acc += accum106_0(5);
    acc += guard106_0(acc);
    acc += pick106_0_0(acc, acc + 1);
    acc += pick106_0_1(acc, acc + 2);
    acc += pick106_0_2(acc, acc + 1);
    S106_1 s1 = mk106_1(acc);
    bump106_1(&s1, 4);
    acc += probe106_1(&s1);
    acc += read106_1(&s1);
    acc += classify106_1(1, acc, acc);
    acc += accum106_1(6);
    acc += guard106_1(acc);
    acc += pick106_1_0(acc, acc + 7);
    acc += pick106_1_1(acc, acc + 8);
    S106_2 s2 = mk106_2(acc);
    bump106_2(&s2, 6);
    acc += probe106_2(&s2);
    acc += read106_2(&s2);
    acc += classify106_2(1, acc, acc);
    acc += accum106_2(3);
    acc += guard106_2(acc);
    acc += pick106_2_0(acc, acc + 9);
    acc += pick106_2_1(acc, acc + 6);
    acc += pick106_2_2(acc, acc + 9);
    return clampi(acc);
}
