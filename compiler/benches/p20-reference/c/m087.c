/* GENERATED C mirror of reference module m087. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S87_0;

static S87_0 mk87_0(long a) {
    S87_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe87_0(const S87_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read87_0(const S87_0 *s) {
    return s->a * 3;
}
static void bump87_0(S87_0 *s, long d) {
    s->a = s->a + d;
}
static long classify87_0(int tag, long a, long b) {
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
static long accum87_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard87_0(long x) {
    return x + 4;
}

static long pick87_0_0(long a, long b) { return a > b ? a : b; }
static long pick87_0_1(long a, long b) { return a > b ? a : b; }
static long pick87_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S87_1;

static S87_1 mk87_1(long a) {
    S87_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe87_1(const S87_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read87_1(const S87_1 *s) {
    return s->a * 4;
}
static void bump87_1(S87_1 *s, long d) {
    s->a = s->a + d;
}
static long classify87_1(int tag, long a, long b) {
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
static long accum87_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard87_1(long x) {
    return x + 4;
}

static long pick87_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S87_2;

static S87_2 mk87_2(long a) {
    S87_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe87_2(const S87_2 *s) {
    return s->a + s->n0;
}
static long read87_2(const S87_2 *s) {
    return s->a * 6;
}
static void bump87_2(S87_2 *s, long d) {
    s->a = s->a + d;
}
static long classify87_2(int tag, long a, long b) {
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
static long accum87_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard87_2(long x) {
    return x + 6;
}

static long pick87_2_0(long a, long b) { return a > b ? a : b; }
long f087(long x) {
    long acc = x;
    acc += f009(x + 1);
    acc += f011(x + 2);
    acc += f023(x + 3);
    S87_0 s0 = mk87_0(acc);
    bump87_0(&s0, 8);
    acc += probe87_0(&s0);
    acc += read87_0(&s0);
    acc += classify87_0(1, acc, acc);
    acc += accum87_0(6);
    acc += guard87_0(acc);
    acc += pick87_0_0(acc, acc + 3);
    acc += pick87_0_1(acc, acc + 8);
    acc += pick87_0_2(acc, acc + 9);
    S87_1 s1 = mk87_1(acc);
    bump87_1(&s1, 5);
    acc += probe87_1(&s1);
    acc += read87_1(&s1);
    acc += classify87_1(1, acc, acc);
    acc += accum87_1(4);
    acc += guard87_1(acc);
    acc += pick87_1_0(acc, acc + 9);
    S87_2 s2 = mk87_2(acc);
    bump87_2(&s2, 2);
    acc += probe87_2(&s2);
    acc += read87_2(&s2);
    acc += classify87_2(1, acc, acc);
    acc += accum87_2(7);
    acc += guard87_2(acc);
    acc += pick87_2_0(acc, acc + 6);
    return clampi(acc);
}
