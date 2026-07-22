/* GENERATED C mirror of reference module m111. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S111_0;

static S111_0 mk111_0(long a) {
    S111_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe111_0(const S111_0 *s) {
    return s->a + s->n0;
}
static long read111_0(const S111_0 *s) {
    return s->a * 2;
}
static void bump111_0(S111_0 *s, long d) {
    s->a = s->a + d;
}
static long classify111_0(int tag, long a, long b) {
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
static long accum111_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard111_0(long x) {
    return x + 8;
}

static long pick111_0_0(long a, long b) { return a > b ? a : b; }
static long pick111_0_1(long a, long b) { return a > b ? a : b; }
static long pick111_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S111_1;

static S111_1 mk111_1(long a) {
    S111_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe111_1(const S111_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read111_1(const S111_1 *s) {
    return s->a * 6;
}
static void bump111_1(S111_1 *s, long d) {
    s->a = s->a + d;
}
static long classify111_1(int tag, long a, long b) {
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
static long accum111_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard111_1(long x) {
    return x + 1;
}

static long pick111_1_0(long a, long b) { return a > b ? a : b; }
static long pick111_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S111_2;

static S111_2 mk111_2(long a) {
    S111_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe111_2(const S111_2 *s) {
    return s->a + s->n0;
}
static long read111_2(const S111_2 *s) {
    return s->a * 3;
}
static void bump111_2(S111_2 *s, long d) {
    s->a = s->a + d;
}
static long classify111_2(int tag, long a, long b) {
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
static long accum111_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard111_2(long x) {
    return x + 3;
}

static long pick111_2_0(long a, long b) { return a > b ? a : b; }
static long pick111_2_1(long a, long b) { return a > b ? a : b; }
long f111(long x) {
    long acc = x;
    acc += f049(x + 1);
    S111_0 s0 = mk111_0(acc);
    bump111_0(&s0, 3);
    acc += probe111_0(&s0);
    acc += read111_0(&s0);
    acc += classify111_0(1, acc, acc);
    acc += accum111_0(9);
    acc += guard111_0(acc);
    acc += pick111_0_0(acc, acc + 1);
    acc += pick111_0_1(acc, acc + 4);
    acc += pick111_0_2(acc, acc + 8);
    S111_1 s1 = mk111_1(acc);
    bump111_1(&s1, 6);
    acc += probe111_1(&s1);
    acc += read111_1(&s1);
    acc += classify111_1(1, acc, acc);
    acc += accum111_1(9);
    acc += guard111_1(acc);
    acc += pick111_1_0(acc, acc + 9);
    acc += pick111_1_1(acc, acc + 9);
    S111_2 s2 = mk111_2(acc);
    bump111_2(&s2, 9);
    acc += probe111_2(&s2);
    acc += read111_2(&s2);
    acc += classify111_2(1, acc, acc);
    acc += accum111_2(9);
    acc += guard111_2(acc);
    acc += pick111_2_0(acc, acc + 4);
    acc += pick111_2_1(acc, acc + 1);
    return clampi(acc);
}
