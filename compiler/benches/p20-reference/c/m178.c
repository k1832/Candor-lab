/* GENERATED C mirror of reference module m178. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S178_0;

static S178_0 mk178_0(long a) {
    S178_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe178_0(const S178_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read178_0(const S178_0 *s) {
    return s->a * 4;
}
static void bump178_0(S178_0 *s, long d) {
    s->a = s->a + d;
}
static long classify178_0(int tag, long a, long b) {
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
static long accum178_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard178_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S178_1;

static S178_1 mk178_1(long a) {
    S178_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe178_1(const S178_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read178_1(const S178_1 *s) {
    return s->a * 2;
}
static void bump178_1(S178_1 *s, long d) {
    s->a = s->a + d;
}
static long classify178_1(int tag, long a, long b) {
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
static long accum178_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard178_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S178_2;

static S178_2 mk178_2(long a) {
    S178_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe178_2(const S178_2 *s) {
    return s->a + s->n0;
}
static long read178_2(const S178_2 *s) {
    return s->a * 5;
}
static void bump178_2(S178_2 *s, long d) {
    s->a = s->a + d;
}
static long classify178_2(int tag, long a, long b) {
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
static long accum178_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard178_2(long x) {
    return x + 1;
}

long f178(long x) {
    long acc = x;
    acc += f011(x + 1);
    acc += f101(x + 2);
    acc += f102(x + 3);
    S178_0 s0 = mk178_0(acc);
    bump178_0(&s0, 3);
    acc += probe178_0(&s0);
    acc += read178_0(&s0);
    acc += classify178_0(1, acc, acc);
    acc += accum178_0(7);
    acc += guard178_0(acc);
    S178_1 s1 = mk178_1(acc);
    bump178_1(&s1, 5);
    acc += probe178_1(&s1);
    acc += read178_1(&s1);
    acc += classify178_1(1, acc, acc);
    acc += accum178_1(4);
    acc += guard178_1(acc);
    S178_2 s2 = mk178_2(acc);
    bump178_2(&s2, 7);
    acc += probe178_2(&s2);
    acc += read178_2(&s2);
    acc += classify178_2(1, acc, acc);
    acc += accum178_2(8);
    acc += guard178_2(acc);
    return clampi(acc);
}
