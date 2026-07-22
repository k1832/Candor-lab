/* GENERATED C mirror of reference module m090. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S90_0;

static S90_0 mk90_0(long a) {
    S90_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe90_0(const S90_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read90_0(const S90_0 *s) {
    return s->a * 5;
}
static void bump90_0(S90_0 *s, long d) {
    s->a = s->a + d;
}
static long classify90_0(int tag, long a, long b) {
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
static long accum90_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard90_0(long x) {
    return x + 1;
}

static long pick90_0_0(long a, long b) { return a > b ? a : b; }
static long pick90_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S90_1;

static S90_1 mk90_1(long a) {
    S90_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe90_1(const S90_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read90_1(const S90_1 *s) {
    return s->a * 4;
}
static void bump90_1(S90_1 *s, long d) {
    s->a = s->a + d;
}
static long classify90_1(int tag, long a, long b) {
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
static long accum90_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard90_1(long x) {
    return x + 1;
}

static long pick90_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S90_2;

static S90_2 mk90_2(long a) {
    S90_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe90_2(const S90_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read90_2(const S90_2 *s) {
    return s->a * 4;
}
static void bump90_2(S90_2 *s, long d) {
    s->a = s->a + d;
}
static long classify90_2(int tag, long a, long b) {
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
static long accum90_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard90_2(long x) {
    return x + 2;
}

static long pick90_2_0(long a, long b) { return a > b ? a : b; }
static long pick90_2_1(long a, long b) { return a > b ? a : b; }
long f090(long x) {
    long acc = x;
    acc += f012(x + 1);
    acc += f027(x + 2);
    acc += f062(x + 3);
    S90_0 s0 = mk90_0(acc);
    bump90_0(&s0, 5);
    acc += probe90_0(&s0);
    acc += read90_0(&s0);
    acc += classify90_0(1, acc, acc);
    acc += accum90_0(6);
    acc += guard90_0(acc);
    acc += pick90_0_0(acc, acc + 9);
    acc += pick90_0_1(acc, acc + 7);
    S90_1 s1 = mk90_1(acc);
    bump90_1(&s1, 1);
    acc += probe90_1(&s1);
    acc += read90_1(&s1);
    acc += classify90_1(1, acc, acc);
    acc += accum90_1(9);
    acc += guard90_1(acc);
    acc += pick90_1_0(acc, acc + 3);
    S90_2 s2 = mk90_2(acc);
    bump90_2(&s2, 6);
    acc += probe90_2(&s2);
    acc += read90_2(&s2);
    acc += classify90_2(1, acc, acc);
    acc += accum90_2(5);
    acc += guard90_2(acc);
    acc += pick90_2_0(acc, acc + 2);
    acc += pick90_2_1(acc, acc + 5);
    return clampi(acc);
}
