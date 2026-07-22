/* GENERATED C mirror of reference module m161. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S161_0;

static S161_0 mk161_0(long a) {
    S161_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe161_0(const S161_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read161_0(const S161_0 *s) {
    return s->a * 2;
}
static void bump161_0(S161_0 *s, long d) {
    s->a = s->a + d;
}
static long classify161_0(int tag, long a, long b) {
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
static long accum161_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard161_0(long x) {
    return x + 9;
}

static long pick161_0_0(long a, long b) { return a > b ? a : b; }
static long pick161_0_1(long a, long b) { return a > b ? a : b; }
static long pick161_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S161_1;

static S161_1 mk161_1(long a) {
    S161_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe161_1(const S161_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read161_1(const S161_1 *s) {
    return s->a * 7;
}
static void bump161_1(S161_1 *s, long d) {
    s->a = s->a + d;
}
static long classify161_1(int tag, long a, long b) {
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
static long accum161_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard161_1(long x) {
    return x + 3;
}

static long pick161_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S161_2;

static S161_2 mk161_2(long a) {
    S161_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe161_2(const S161_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read161_2(const S161_2 *s) {
    return s->a * 4;
}
static void bump161_2(S161_2 *s, long d) {
    s->a = s->a + d;
}
static long classify161_2(int tag, long a, long b) {
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
static long accum161_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard161_2(long x) {
    return x + 9;
}

static long pick161_2_0(long a, long b) { return a > b ? a : b; }
long f161(long x) {
    long acc = x;
    acc += f045(x + 1);
    acc += f116(x + 2);
    acc += f141(x + 3);
    S161_0 s0 = mk161_0(acc);
    bump161_0(&s0, 8);
    acc += probe161_0(&s0);
    acc += read161_0(&s0);
    acc += classify161_0(1, acc, acc);
    acc += accum161_0(3);
    acc += guard161_0(acc);
    acc += pick161_0_0(acc, acc + 4);
    acc += pick161_0_1(acc, acc + 9);
    acc += pick161_0_2(acc, acc + 6);
    S161_1 s1 = mk161_1(acc);
    bump161_1(&s1, 7);
    acc += probe161_1(&s1);
    acc += read161_1(&s1);
    acc += classify161_1(1, acc, acc);
    acc += accum161_1(9);
    acc += guard161_1(acc);
    acc += pick161_1_0(acc, acc + 8);
    S161_2 s2 = mk161_2(acc);
    bump161_2(&s2, 5);
    acc += probe161_2(&s2);
    acc += read161_2(&s2);
    acc += classify161_2(1, acc, acc);
    acc += accum161_2(6);
    acc += guard161_2(acc);
    acc += pick161_2_0(acc, acc + 1);
    return clampi(acc);
}
