/* GENERATED C mirror of reference module m088. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S88_0;

static S88_0 mk88_0(long a) {
    S88_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe88_0(const S88_0 *s) {
    return s->a + s->n0;
}
static long read88_0(const S88_0 *s) {
    return s->a * 5;
}
static void bump88_0(S88_0 *s, long d) {
    s->a = s->a + d;
}
static long classify88_0(int tag, long a, long b) {
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
static long accum88_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard88_0(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S88_1;

static S88_1 mk88_1(long a) {
    S88_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe88_1(const S88_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read88_1(const S88_1 *s) {
    return s->a * 2;
}
static void bump88_1(S88_1 *s, long d) {
    s->a = s->a + d;
}
static long classify88_1(int tag, long a, long b) {
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
static long accum88_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard88_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S88_2;

static S88_2 mk88_2(long a) {
    S88_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe88_2(const S88_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read88_2(const S88_2 *s) {
    return s->a * 6;
}
static void bump88_2(S88_2 *s, long d) {
    s->a = s->a + d;
}
static long classify88_2(int tag, long a, long b) {
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
static long accum88_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard88_2(long x) {
    return x + 7;
}

long f088(long x) {
    long acc = x;
    acc += f014(x + 1);
    acc += f017(x + 2);
    S88_0 s0 = mk88_0(acc);
    bump88_0(&s0, 8);
    acc += probe88_0(&s0);
    acc += read88_0(&s0);
    acc += classify88_0(1, acc, acc);
    acc += accum88_0(7);
    acc += guard88_0(acc);
    S88_1 s1 = mk88_1(acc);
    bump88_1(&s1, 4);
    acc += probe88_1(&s1);
    acc += read88_1(&s1);
    acc += classify88_1(1, acc, acc);
    acc += accum88_1(5);
    acc += guard88_1(acc);
    S88_2 s2 = mk88_2(acc);
    bump88_2(&s2, 4);
    acc += probe88_2(&s2);
    acc += read88_2(&s2);
    acc += classify88_2(1, acc, acc);
    acc += accum88_2(4);
    acc += guard88_2(acc);
    return clampi(acc);
}
