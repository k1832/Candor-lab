/* GENERATED C mirror of reference module m141. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S141_0;

static S141_0 mk141_0(long a) {
    S141_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe141_0(const S141_0 *s) {
    return s->a + s->n0;
}
static long read141_0(const S141_0 *s) {
    return s->a * 4;
}
static void bump141_0(S141_0 *s, long d) {
    s->a = s->a + d;
}
static long classify141_0(int tag, long a, long b) {
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
static long accum141_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard141_0(long x) {
    return x + 1;
}

static long pick141_0_0(long a, long b) { return a > b ? a : b; }
static long pick141_0_1(long a, long b) { return a > b ? a : b; }
static long pick141_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S141_1;

static S141_1 mk141_1(long a) {
    S141_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe141_1(const S141_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read141_1(const S141_1 *s) {
    return s->a * 4;
}
static void bump141_1(S141_1 *s, long d) {
    s->a = s->a + d;
}
static long classify141_1(int tag, long a, long b) {
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
static long accum141_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard141_1(long x) {
    return x + 5;
}

static long pick141_1_0(long a, long b) { return a > b ? a : b; }
static long pick141_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S141_2;

static S141_2 mk141_2(long a) {
    S141_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe141_2(const S141_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read141_2(const S141_2 *s) {
    return s->a * 3;
}
static void bump141_2(S141_2 *s, long d) {
    s->a = s->a + d;
}
static long classify141_2(int tag, long a, long b) {
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
static long accum141_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard141_2(long x) {
    return x + 1;
}

static long pick141_2_0(long a, long b) { return a > b ? a : b; }
static long pick141_2_1(long a, long b) { return a > b ? a : b; }
static long pick141_2_2(long a, long b) { return a > b ? a : b; }
long f141(long x) {
    long acc = x;
    acc += f075(x + 1);
    S141_0 s0 = mk141_0(acc);
    bump141_0(&s0, 6);
    acc += probe141_0(&s0);
    acc += read141_0(&s0);
    acc += classify141_0(1, acc, acc);
    acc += accum141_0(4);
    acc += guard141_0(acc);
    acc += pick141_0_0(acc, acc + 9);
    acc += pick141_0_1(acc, acc + 5);
    acc += pick141_0_2(acc, acc + 6);
    S141_1 s1 = mk141_1(acc);
    bump141_1(&s1, 9);
    acc += probe141_1(&s1);
    acc += read141_1(&s1);
    acc += classify141_1(1, acc, acc);
    acc += accum141_1(3);
    acc += guard141_1(acc);
    acc += pick141_1_0(acc, acc + 2);
    acc += pick141_1_1(acc, acc + 2);
    S141_2 s2 = mk141_2(acc);
    bump141_2(&s2, 5);
    acc += probe141_2(&s2);
    acc += read141_2(&s2);
    acc += classify141_2(1, acc, acc);
    acc += accum141_2(8);
    acc += guard141_2(acc);
    acc += pick141_2_0(acc, acc + 4);
    acc += pick141_2_1(acc, acc + 5);
    acc += pick141_2_2(acc, acc + 8);
    return clampi(acc);
}
