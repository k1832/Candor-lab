/* GENERATED C mirror of reference module m033. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S33_0;

static S33_0 mk33_0(long a) {
    S33_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe33_0(const S33_0 *s) {
    return s->a + s->n0;
}
static long read33_0(const S33_0 *s) {
    return s->a * 3;
}
static void bump33_0(S33_0 *s, long d) {
    s->a = s->a + d;
}
static long classify33_0(int tag, long a, long b) {
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
static long accum33_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard33_0(long x) {
    return x + 9;
}

static long pick33_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S33_1;

static S33_1 mk33_1(long a) {
    S33_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe33_1(const S33_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read33_1(const S33_1 *s) {
    return s->a * 2;
}
static void bump33_1(S33_1 *s, long d) {
    s->a = s->a + d;
}
static long classify33_1(int tag, long a, long b) {
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
static long accum33_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard33_1(long x) {
    return x + 1;
}

static long pick33_1_0(long a, long b) { return a > b ? a : b; }
static long pick33_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S33_2;

static S33_2 mk33_2(long a) {
    S33_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe33_2(const S33_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read33_2(const S33_2 *s) {
    return s->a * 7;
}
static void bump33_2(S33_2 *s, long d) {
    s->a = s->a + d;
}
static long classify33_2(int tag, long a, long b) {
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
static long accum33_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard33_2(long x) {
    return x + 5;
}

static long pick33_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S33_3;

static S33_3 mk33_3(long a) {
    S33_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe33_3(const S33_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read33_3(const S33_3 *s) {
    return s->a * 5;
}
static void bump33_3(S33_3 *s, long d) {
    s->a = s->a + d;
}
static long classify33_3(int tag, long a, long b) {
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
static long accum33_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard33_3(long x) {
    return x + 4;
}

static long pick33_3_0(long a, long b) { return a > b ? a : b; }
static long pick33_3_1(long a, long b) { return a > b ? a : b; }
static long pick33_3_2(long a, long b) { return a > b ? a : b; }
long f033(long x) {
    long acc = x;
    acc += f000(x + 1);
    acc += f019(x + 2);
    S33_0 s0 = mk33_0(acc);
    bump33_0(&s0, 7);
    acc += probe33_0(&s0);
    acc += read33_0(&s0);
    acc += classify33_0(1, acc, acc);
    acc += accum33_0(8);
    acc += guard33_0(acc);
    acc += pick33_0_0(acc, acc + 6);
    S33_1 s1 = mk33_1(acc);
    bump33_1(&s1, 5);
    acc += probe33_1(&s1);
    acc += read33_1(&s1);
    acc += classify33_1(1, acc, acc);
    acc += accum33_1(5);
    acc += guard33_1(acc);
    acc += pick33_1_0(acc, acc + 6);
    acc += pick33_1_1(acc, acc + 1);
    S33_2 s2 = mk33_2(acc);
    bump33_2(&s2, 3);
    acc += probe33_2(&s2);
    acc += read33_2(&s2);
    acc += classify33_2(1, acc, acc);
    acc += accum33_2(7);
    acc += guard33_2(acc);
    acc += pick33_2_0(acc, acc + 4);
    S33_3 s3 = mk33_3(acc);
    bump33_3(&s3, 4);
    acc += probe33_3(&s3);
    acc += read33_3(&s3);
    acc += classify33_3(1, acc, acc);
    acc += accum33_3(4);
    acc += guard33_3(acc);
    acc += pick33_3_0(acc, acc + 7);
    acc += pick33_3_1(acc, acc + 4);
    acc += pick33_3_2(acc, acc + 6);
    return clampi(acc);
}
