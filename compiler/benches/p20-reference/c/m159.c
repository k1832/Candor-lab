/* GENERATED C mirror of reference module m159. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S159_0;

static S159_0 mk159_0(long a) {
    S159_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe159_0(const S159_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read159_0(const S159_0 *s) {
    return s->a * 3;
}
static void bump159_0(S159_0 *s, long d) {
    s->a = s->a + d;
}
static long classify159_0(int tag, long a, long b) {
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
static long accum159_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard159_0(long x) {
    return x + 1;
}

static long pick159_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S159_1;

static S159_1 mk159_1(long a) {
    S159_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe159_1(const S159_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read159_1(const S159_1 *s) {
    return s->a * 5;
}
static void bump159_1(S159_1 *s, long d) {
    s->a = s->a + d;
}
static long classify159_1(int tag, long a, long b) {
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
static long accum159_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard159_1(long x) {
    return x + 3;
}

static long pick159_1_0(long a, long b) { return a > b ? a : b; }
static long pick159_1_1(long a, long b) { return a > b ? a : b; }
static long pick159_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S159_2;

static S159_2 mk159_2(long a) {
    S159_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe159_2(const S159_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read159_2(const S159_2 *s) {
    return s->a * 2;
}
static void bump159_2(S159_2 *s, long d) {
    s->a = s->a + d;
}
static long classify159_2(int tag, long a, long b) {
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
static long accum159_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard159_2(long x) {
    return x + 1;
}

static long pick159_2_0(long a, long b) { return a > b ? a : b; }
long f159(long x) {
    long acc = x;
    acc += f033(x + 1);
    acc += f043(x + 2);
    acc += f090(x + 3);
    acc += f131(x + 4);
    S159_0 s0 = mk159_0(acc);
    bump159_0(&s0, 5);
    acc += probe159_0(&s0);
    acc += read159_0(&s0);
    acc += classify159_0(1, acc, acc);
    acc += accum159_0(6);
    acc += guard159_0(acc);
    acc += pick159_0_0(acc, acc + 1);
    S159_1 s1 = mk159_1(acc);
    bump159_1(&s1, 9);
    acc += probe159_1(&s1);
    acc += read159_1(&s1);
    acc += classify159_1(1, acc, acc);
    acc += accum159_1(9);
    acc += guard159_1(acc);
    acc += pick159_1_0(acc, acc + 5);
    acc += pick159_1_1(acc, acc + 9);
    acc += pick159_1_2(acc, acc + 1);
    S159_2 s2 = mk159_2(acc);
    bump159_2(&s2, 3);
    acc += probe159_2(&s2);
    acc += read159_2(&s2);
    acc += classify159_2(1, acc, acc);
    acc += accum159_2(6);
    acc += guard159_2(acc);
    acc += pick159_2_0(acc, acc + 1);
    return clampi(acc);
}
