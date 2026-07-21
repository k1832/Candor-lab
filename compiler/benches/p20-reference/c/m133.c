/* GENERATED C mirror of reference module m133. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S133_0;

static S133_0 mk133_0(long a) {
    S133_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe133_0(const S133_0 *s) {
    return s->a + s->n0;
}
static long read133_0(const S133_0 *s) {
    return s->a * 6;
}
static void bump133_0(S133_0 *s, long d) {
    s->a = s->a + d;
}
static long classify133_0(int tag, long a, long b) {
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
static long accum133_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard133_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S133_1;

static S133_1 mk133_1(long a) {
    S133_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe133_1(const S133_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read133_1(const S133_1 *s) {
    return s->a * 2;
}
static void bump133_1(S133_1 *s, long d) {
    s->a = s->a + d;
}
static long classify133_1(int tag, long a, long b) {
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
static long accum133_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard133_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S133_2;

static S133_2 mk133_2(long a) {
    S133_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe133_2(const S133_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read133_2(const S133_2 *s) {
    return s->a * 4;
}
static void bump133_2(S133_2 *s, long d) {
    s->a = s->a + d;
}
static long classify133_2(int tag, long a, long b) {
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
static long accum133_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard133_2(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S133_3;

static S133_3 mk133_3(long a) {
    S133_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe133_3(const S133_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read133_3(const S133_3 *s) {
    return s->a * 5;
}
static void bump133_3(S133_3 *s, long d) {
    s->a = s->a + d;
}
static long classify133_3(int tag, long a, long b) {
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
static long accum133_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard133_3(long x) {
    return x + 4;
}

long f133(long x) {
    long acc = x;
    acc += f095(x + 1);
    S133_0 s0 = mk133_0(acc);
    bump133_0(&s0, 1);
    acc += probe133_0(&s0);
    acc += read133_0(&s0);
    acc += classify133_0(1, acc, acc);
    acc += accum133_0(4);
    acc += guard133_0(acc);
    S133_1 s1 = mk133_1(acc);
    bump133_1(&s1, 5);
    acc += probe133_1(&s1);
    acc += read133_1(&s1);
    acc += classify133_1(1, acc, acc);
    acc += accum133_1(3);
    acc += guard133_1(acc);
    S133_2 s2 = mk133_2(acc);
    bump133_2(&s2, 3);
    acc += probe133_2(&s2);
    acc += read133_2(&s2);
    acc += classify133_2(1, acc, acc);
    acc += accum133_2(7);
    acc += guard133_2(acc);
    S133_3 s3 = mk133_3(acc);
    bump133_3(&s3, 8);
    acc += probe133_3(&s3);
    acc += read133_3(&s3);
    acc += classify133_3(1, acc, acc);
    acc += accum133_3(7);
    acc += guard133_3(acc);
    return clampi(acc);
}
