/* GENERATED C mirror of reference module m128. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S128_0;

static S128_0 mk128_0(long a) {
    S128_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe128_0(const S128_0 *s) {
    return s->a + s->n0;
}
static long read128_0(const S128_0 *s) {
    return s->a * 2;
}
static void bump128_0(S128_0 *s, long d) {
    s->a = s->a + d;
}
static long classify128_0(int tag, long a, long b) {
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
static long accum128_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard128_0(long x) {
    return x + 3;
}

static long pick128_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S128_1;

static S128_1 mk128_1(long a) {
    S128_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe128_1(const S128_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read128_1(const S128_1 *s) {
    return s->a * 2;
}
static void bump128_1(S128_1 *s, long d) {
    s->a = s->a + d;
}
static long classify128_1(int tag, long a, long b) {
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
static long accum128_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard128_1(long x) {
    return x + 9;
}

static long pick128_1_0(long a, long b) { return a > b ? a : b; }
static long pick128_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S128_2;

static S128_2 mk128_2(long a) {
    S128_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe128_2(const S128_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read128_2(const S128_2 *s) {
    return s->a * 3;
}
static void bump128_2(S128_2 *s, long d) {
    s->a = s->a + d;
}
static long classify128_2(int tag, long a, long b) {
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
static long accum128_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard128_2(long x) {
    return x + 3;
}

static long pick128_2_0(long a, long b) { return a > b ? a : b; }
static long pick128_2_1(long a, long b) { return a > b ? a : b; }
static long pick128_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S128_3;

static S128_3 mk128_3(long a) {
    S128_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe128_3(const S128_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read128_3(const S128_3 *s) {
    return s->a * 6;
}
static void bump128_3(S128_3 *s, long d) {
    s->a = s->a + d;
}
static long classify128_3(int tag, long a, long b) {
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
static long accum128_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard128_3(long x) {
    return x + 1;
}

static long pick128_3_0(long a, long b) { return a > b ? a : b; }
static long pick128_3_1(long a, long b) { return a > b ? a : b; }
static long pick128_3_2(long a, long b) { return a > b ? a : b; }
long f128(long x) {
    long acc = x;
    acc += f002(x + 1);
    acc += f018(x + 2);
    acc += f061(x + 3);
    acc += f079(x + 4);
    S128_0 s0 = mk128_0(acc);
    bump128_0(&s0, 9);
    acc += probe128_0(&s0);
    acc += read128_0(&s0);
    acc += classify128_0(1, acc, acc);
    acc += accum128_0(4);
    acc += guard128_0(acc);
    acc += pick128_0_0(acc, acc + 3);
    S128_1 s1 = mk128_1(acc);
    bump128_1(&s1, 4);
    acc += probe128_1(&s1);
    acc += read128_1(&s1);
    acc += classify128_1(1, acc, acc);
    acc += accum128_1(5);
    acc += guard128_1(acc);
    acc += pick128_1_0(acc, acc + 6);
    acc += pick128_1_1(acc, acc + 9);
    S128_2 s2 = mk128_2(acc);
    bump128_2(&s2, 1);
    acc += probe128_2(&s2);
    acc += read128_2(&s2);
    acc += classify128_2(1, acc, acc);
    acc += accum128_2(7);
    acc += guard128_2(acc);
    acc += pick128_2_0(acc, acc + 1);
    acc += pick128_2_1(acc, acc + 9);
    acc += pick128_2_2(acc, acc + 3);
    S128_3 s3 = mk128_3(acc);
    bump128_3(&s3, 2);
    acc += probe128_3(&s3);
    acc += read128_3(&s3);
    acc += classify128_3(1, acc, acc);
    acc += accum128_3(8);
    acc += guard128_3(acc);
    acc += pick128_3_0(acc, acc + 1);
    acc += pick128_3_1(acc, acc + 5);
    acc += pick128_3_2(acc, acc + 3);
    return clampi(acc);
}
