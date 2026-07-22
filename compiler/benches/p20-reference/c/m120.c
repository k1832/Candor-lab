/* GENERATED C mirror of reference module m120. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S120_0;

static S120_0 mk120_0(long a) {
    S120_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe120_0(const S120_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read120_0(const S120_0 *s) {
    return s->a * 5;
}
static void bump120_0(S120_0 *s, long d) {
    s->a = s->a + d;
}
static long classify120_0(int tag, long a, long b) {
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
static long accum120_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard120_0(long x) {
    return x + 8;
}

static long pick120_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S120_1;

static S120_1 mk120_1(long a) {
    S120_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe120_1(const S120_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read120_1(const S120_1 *s) {
    return s->a * 5;
}
static void bump120_1(S120_1 *s, long d) {
    s->a = s->a + d;
}
static long classify120_1(int tag, long a, long b) {
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
static long accum120_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard120_1(long x) {
    return x + 3;
}

static long pick120_1_0(long a, long b) { return a > b ? a : b; }
static long pick120_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S120_2;

static S120_2 mk120_2(long a) {
    S120_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe120_2(const S120_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read120_2(const S120_2 *s) {
    return s->a * 4;
}
static void bump120_2(S120_2 *s, long d) {
    s->a = s->a + d;
}
static long classify120_2(int tag, long a, long b) {
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
static long accum120_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard120_2(long x) {
    return x + 6;
}

static long pick120_2_0(long a, long b) { return a > b ? a : b; }
static long pick120_2_1(long a, long b) { return a > b ? a : b; }
static long pick120_2_2(long a, long b) { return a > b ? a : b; }
long f120(long x) {
    long acc = x;
    acc += f012(x + 1);
    acc += f063(x + 2);
    acc += f091(x + 3);
    acc += f100(x + 4);
    S120_0 s0 = mk120_0(acc);
    bump120_0(&s0, 5);
    acc += probe120_0(&s0);
    acc += read120_0(&s0);
    acc += classify120_0(1, acc, acc);
    acc += accum120_0(3);
    acc += guard120_0(acc);
    acc += pick120_0_0(acc, acc + 2);
    S120_1 s1 = mk120_1(acc);
    bump120_1(&s1, 7);
    acc += probe120_1(&s1);
    acc += read120_1(&s1);
    acc += classify120_1(1, acc, acc);
    acc += accum120_1(8);
    acc += guard120_1(acc);
    acc += pick120_1_0(acc, acc + 3);
    acc += pick120_1_1(acc, acc + 2);
    S120_2 s2 = mk120_2(acc);
    bump120_2(&s2, 1);
    acc += probe120_2(&s2);
    acc += read120_2(&s2);
    acc += classify120_2(1, acc, acc);
    acc += accum120_2(4);
    acc += guard120_2(acc);
    acc += pick120_2_0(acc, acc + 1);
    acc += pick120_2_1(acc, acc + 4);
    acc += pick120_2_2(acc, acc + 5);
    return clampi(acc);
}
