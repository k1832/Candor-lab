/* GENERATED C mirror of reference module m023. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S23_0;

static S23_0 mk23_0(long a) {
    S23_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe23_0(const S23_0 *s) {
    return s->a + s->n0;
}
static long read23_0(const S23_0 *s) {
    return s->a * 2;
}
static void bump23_0(S23_0 *s, long d) {
    s->a = s->a + d;
}
static long classify23_0(int tag, long a, long b) {
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
static long accum23_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard23_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S23_1;

static S23_1 mk23_1(long a) {
    S23_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe23_1(const S23_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read23_1(const S23_1 *s) {
    return s->a * 7;
}
static void bump23_1(S23_1 *s, long d) {
    s->a = s->a + d;
}
static long classify23_1(int tag, long a, long b) {
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
static long accum23_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard23_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S23_2;

static S23_2 mk23_2(long a) {
    S23_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe23_2(const S23_2 *s) {
    return s->a + s->n0;
}
static long read23_2(const S23_2 *s) {
    return s->a * 3;
}
static void bump23_2(S23_2 *s, long d) {
    s->a = s->a + d;
}
static long classify23_2(int tag, long a, long b) {
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
static long accum23_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard23_2(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S23_3;

static S23_3 mk23_3(long a) {
    S23_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe23_3(const S23_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read23_3(const S23_3 *s) {
    return s->a * 7;
}
static void bump23_3(S23_3 *s, long d) {
    s->a = s->a + d;
}
static long classify23_3(int tag, long a, long b) {
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
static long accum23_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard23_3(long x) {
    return x + 1;
}

long f023(long x) {
    long acc = x;
    acc += f007(x + 1);
    S23_0 s0 = mk23_0(acc);
    bump23_0(&s0, 6);
    acc += probe23_0(&s0);
    acc += read23_0(&s0);
    acc += classify23_0(1, acc, acc);
    acc += accum23_0(8);
    acc += guard23_0(acc);
    S23_1 s1 = mk23_1(acc);
    bump23_1(&s1, 6);
    acc += probe23_1(&s1);
    acc += read23_1(&s1);
    acc += classify23_1(1, acc, acc);
    acc += accum23_1(4);
    acc += guard23_1(acc);
    S23_2 s2 = mk23_2(acc);
    bump23_2(&s2, 7);
    acc += probe23_2(&s2);
    acc += read23_2(&s2);
    acc += classify23_2(1, acc, acc);
    acc += accum23_2(8);
    acc += guard23_2(acc);
    S23_3 s3 = mk23_3(acc);
    bump23_3(&s3, 6);
    acc += probe23_3(&s3);
    acc += read23_3(&s3);
    acc += classify23_3(1, acc, acc);
    acc += accum23_3(7);
    acc += guard23_3(acc);
    return clampi(acc);
}
