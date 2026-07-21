/* GENERATED C mirror of reference module m126. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S126_0;

static S126_0 mk126_0(long a) {
    S126_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe126_0(const S126_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read126_0(const S126_0 *s) {
    return s->a * 6;
}
static void bump126_0(S126_0 *s, long d) {
    s->a = s->a + d;
}
static long classify126_0(int tag, long a, long b) {
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
static long accum126_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard126_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S126_1;

static S126_1 mk126_1(long a) {
    S126_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe126_1(const S126_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read126_1(const S126_1 *s) {
    return s->a * 5;
}
static void bump126_1(S126_1 *s, long d) {
    s->a = s->a + d;
}
static long classify126_1(int tag, long a, long b) {
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
static long accum126_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard126_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S126_2;

static S126_2 mk126_2(long a) {
    S126_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe126_2(const S126_2 *s) {
    return s->a + s->n0;
}
static long read126_2(const S126_2 *s) {
    return s->a * 4;
}
static void bump126_2(S126_2 *s, long d) {
    s->a = s->a + d;
}
static long classify126_2(int tag, long a, long b) {
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
static long accum126_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard126_2(long x) {
    return x + 5;
}

long f126(long x) {
    long acc = x;
    acc += f004(x + 1);
    acc += f059(x + 2);
    acc += f085(x + 3);
    acc += f101(x + 4);
    S126_0 s0 = mk126_0(acc);
    bump126_0(&s0, 2);
    acc += probe126_0(&s0);
    acc += read126_0(&s0);
    acc += classify126_0(1, acc, acc);
    acc += accum126_0(9);
    acc += guard126_0(acc);
    S126_1 s1 = mk126_1(acc);
    bump126_1(&s1, 5);
    acc += probe126_1(&s1);
    acc += read126_1(&s1);
    acc += classify126_1(1, acc, acc);
    acc += accum126_1(5);
    acc += guard126_1(acc);
    S126_2 s2 = mk126_2(acc);
    bump126_2(&s2, 5);
    acc += probe126_2(&s2);
    acc += read126_2(&s2);
    acc += classify126_2(1, acc, acc);
    acc += accum126_2(4);
    acc += guard126_2(acc);
    return clampi(acc);
}
