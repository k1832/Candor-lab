/* GENERATED C mirror of reference module m008. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S8_0;

static S8_0 mk8_0(long a) {
    S8_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe8_0(const S8_0 *s) {
    return s->a + s->n0;
}
static long read8_0(const S8_0 *s) {
    return s->a * 4;
}
static void bump8_0(S8_0 *s, long d) {
    s->a = s->a + d;
}
static long classify8_0(int tag, long a, long b) {
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
static long accum8_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard8_0(long x) {
    return x + 5;
}

static long pick8_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S8_1;

static S8_1 mk8_1(long a) {
    S8_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe8_1(const S8_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read8_1(const S8_1 *s) {
    return s->a * 4;
}
static void bump8_1(S8_1 *s, long d) {
    s->a = s->a + d;
}
static long classify8_1(int tag, long a, long b) {
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
static long accum8_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard8_1(long x) {
    return x + 2;
}

static long pick8_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S8_2;

static S8_2 mk8_2(long a) {
    S8_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe8_2(const S8_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read8_2(const S8_2 *s) {
    return s->a * 6;
}
static void bump8_2(S8_2 *s, long d) {
    s->a = s->a + d;
}
static long classify8_2(int tag, long a, long b) {
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
static long accum8_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard8_2(long x) {
    return x + 8;
}

static long pick8_2_0(long a, long b) { return a > b ? a : b; }
static long pick8_2_1(long a, long b) { return a > b ? a : b; }
static long pick8_2_2(long a, long b) { return a > b ? a : b; }
long f008(long x) {
    long acc = x;
    acc += f006(x + 1);
    S8_0 s0 = mk8_0(acc);
    bump8_0(&s0, 1);
    acc += probe8_0(&s0);
    acc += read8_0(&s0);
    acc += classify8_0(1, acc, acc);
    acc += accum8_0(6);
    acc += guard8_0(acc);
    acc += pick8_0_0(acc, acc + 4);
    S8_1 s1 = mk8_1(acc);
    bump8_1(&s1, 9);
    acc += probe8_1(&s1);
    acc += read8_1(&s1);
    acc += classify8_1(1, acc, acc);
    acc += accum8_1(9);
    acc += guard8_1(acc);
    acc += pick8_1_0(acc, acc + 6);
    S8_2 s2 = mk8_2(acc);
    bump8_2(&s2, 7);
    acc += probe8_2(&s2);
    acc += read8_2(&s2);
    acc += classify8_2(1, acc, acc);
    acc += accum8_2(6);
    acc += guard8_2(acc);
    acc += pick8_2_0(acc, acc + 9);
    acc += pick8_2_1(acc, acc + 6);
    acc += pick8_2_2(acc, acc + 9);
    return clampi(acc);
}
