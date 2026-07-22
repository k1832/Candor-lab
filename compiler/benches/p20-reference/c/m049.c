/* GENERATED C mirror of reference module m049. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S49_0;

static S49_0 mk49_0(long a) {
    S49_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe49_0(const S49_0 *s) {
    return s->a + s->n0;
}
static long read49_0(const S49_0 *s) {
    return s->a * 3;
}
static void bump49_0(S49_0 *s, long d) {
    s->a = s->a + d;
}
static long classify49_0(int tag, long a, long b) {
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
static long accum49_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard49_0(long x) {
    return x + 7;
}

static long pick49_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S49_1;

static S49_1 mk49_1(long a) {
    S49_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe49_1(const S49_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read49_1(const S49_1 *s) {
    return s->a * 2;
}
static void bump49_1(S49_1 *s, long d) {
    s->a = s->a + d;
}
static long classify49_1(int tag, long a, long b) {
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
static long accum49_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard49_1(long x) {
    return x + 7;
}

static long pick49_1_0(long a, long b) { return a > b ? a : b; }
static long pick49_1_1(long a, long b) { return a > b ? a : b; }
static long pick49_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S49_2;

static S49_2 mk49_2(long a) {
    S49_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe49_2(const S49_2 *s) {
    return s->a + s->n0;
}
static long read49_2(const S49_2 *s) {
    return s->a * 5;
}
static void bump49_2(S49_2 *s, long d) {
    s->a = s->a + d;
}
static long classify49_2(int tag, long a, long b) {
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
static long accum49_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard49_2(long x) {
    return x + 4;
}

static long pick49_2_0(long a, long b) { return a > b ? a : b; }
static long pick49_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S49_3;

static S49_3 mk49_3(long a) {
    S49_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe49_3(const S49_3 *s) {
    return s->a + s->n0;
}
static long read49_3(const S49_3 *s) {
    return s->a * 5;
}
static void bump49_3(S49_3 *s, long d) {
    s->a = s->a + d;
}
static long classify49_3(int tag, long a, long b) {
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
static long accum49_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard49_3(long x) {
    return x + 2;
}

static long pick49_3_0(long a, long b) { return a > b ? a : b; }
static long pick49_3_1(long a, long b) { return a > b ? a : b; }
long f049(long x) {
    long acc = x;
    acc += f011(x + 1);
    acc += f021(x + 2);
    acc += f036(x + 3);
    acc += f046(x + 4);
    S49_0 s0 = mk49_0(acc);
    bump49_0(&s0, 7);
    acc += probe49_0(&s0);
    acc += read49_0(&s0);
    acc += classify49_0(1, acc, acc);
    acc += accum49_0(7);
    acc += guard49_0(acc);
    acc += pick49_0_0(acc, acc + 7);
    S49_1 s1 = mk49_1(acc);
    bump49_1(&s1, 4);
    acc += probe49_1(&s1);
    acc += read49_1(&s1);
    acc += classify49_1(1, acc, acc);
    acc += accum49_1(4);
    acc += guard49_1(acc);
    acc += pick49_1_0(acc, acc + 1);
    acc += pick49_1_1(acc, acc + 2);
    acc += pick49_1_2(acc, acc + 2);
    S49_2 s2 = mk49_2(acc);
    bump49_2(&s2, 4);
    acc += probe49_2(&s2);
    acc += read49_2(&s2);
    acc += classify49_2(1, acc, acc);
    acc += accum49_2(7);
    acc += guard49_2(acc);
    acc += pick49_2_0(acc, acc + 1);
    acc += pick49_2_1(acc, acc + 9);
    S49_3 s3 = mk49_3(acc);
    bump49_3(&s3, 4);
    acc += probe49_3(&s3);
    acc += read49_3(&s3);
    acc += classify49_3(1, acc, acc);
    acc += accum49_3(7);
    acc += guard49_3(acc);
    acc += pick49_3_0(acc, acc + 3);
    acc += pick49_3_1(acc, acc + 6);
    return clampi(acc);
}
