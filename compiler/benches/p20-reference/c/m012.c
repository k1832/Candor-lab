/* GENERATED C mirror of reference module m012. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S12_0;

static S12_0 mk12_0(long a) {
    S12_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe12_0(const S12_0 *s) {
    return s->a + s->n0;
}
static long read12_0(const S12_0 *s) {
    return s->a * 3;
}
static void bump12_0(S12_0 *s, long d) {
    s->a = s->a + d;
}
static long classify12_0(int tag, long a, long b) {
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
static long accum12_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard12_0(long x) {
    return x + 1;
}

static long pick12_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S12_1;

static S12_1 mk12_1(long a) {
    S12_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe12_1(const S12_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read12_1(const S12_1 *s) {
    return s->a * 2;
}
static void bump12_1(S12_1 *s, long d) {
    s->a = s->a + d;
}
static long classify12_1(int tag, long a, long b) {
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
static long accum12_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard12_1(long x) {
    return x + 3;
}

static long pick12_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S12_2;

static S12_2 mk12_2(long a) {
    S12_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe12_2(const S12_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read12_2(const S12_2 *s) {
    return s->a * 2;
}
static void bump12_2(S12_2 *s, long d) {
    s->a = s->a + d;
}
static long classify12_2(int tag, long a, long b) {
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
static long accum12_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard12_2(long x) {
    return x + 6;
}

static long pick12_2_0(long a, long b) { return a > b ? a : b; }
long f012(long x) {
    long acc = x;
    acc += f007(x + 1);
    S12_0 s0 = mk12_0(acc);
    bump12_0(&s0, 5);
    acc += probe12_0(&s0);
    acc += read12_0(&s0);
    acc += classify12_0(1, acc, acc);
    acc += accum12_0(7);
    acc += guard12_0(acc);
    acc += pick12_0_0(acc, acc + 3);
    S12_1 s1 = mk12_1(acc);
    bump12_1(&s1, 6);
    acc += probe12_1(&s1);
    acc += read12_1(&s1);
    acc += classify12_1(1, acc, acc);
    acc += accum12_1(5);
    acc += guard12_1(acc);
    acc += pick12_1_0(acc, acc + 3);
    S12_2 s2 = mk12_2(acc);
    bump12_2(&s2, 5);
    acc += probe12_2(&s2);
    acc += read12_2(&s2);
    acc += classify12_2(1, acc, acc);
    acc += accum12_2(4);
    acc += guard12_2(acc);
    acc += pick12_2_0(acc, acc + 4);
    return clampi(acc);
}
