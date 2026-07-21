/* GENERATED C mirror of reference module m041. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S41_0;

static S41_0 mk41_0(long a) {
    S41_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe41_0(const S41_0 *s) {
    return s->a + s->n0;
}
static long read41_0(const S41_0 *s) {
    return s->a * 4;
}
static void bump41_0(S41_0 *s, long d) {
    s->a = s->a + d;
}
static long classify41_0(int tag, long a, long b) {
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
static long accum41_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard41_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S41_1;

static S41_1 mk41_1(long a) {
    S41_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe41_1(const S41_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read41_1(const S41_1 *s) {
    return s->a * 6;
}
static void bump41_1(S41_1 *s, long d) {
    s->a = s->a + d;
}
static long classify41_1(int tag, long a, long b) {
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
static long accum41_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard41_1(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S41_2;

static S41_2 mk41_2(long a) {
    S41_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe41_2(const S41_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read41_2(const S41_2 *s) {
    return s->a * 3;
}
static void bump41_2(S41_2 *s, long d) {
    s->a = s->a + d;
}
static long classify41_2(int tag, long a, long b) {
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
static long accum41_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard41_2(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S41_3;

static S41_3 mk41_3(long a) {
    S41_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe41_3(const S41_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read41_3(const S41_3 *s) {
    return s->a * 2;
}
static void bump41_3(S41_3 *s, long d) {
    s->a = s->a + d;
}
static long classify41_3(int tag, long a, long b) {
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
static long accum41_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard41_3(long x) {
    return x + 4;
}

long f041(long x) {
    long acc = x;
    acc += f010(x + 1);
    acc += f014(x + 2);
    S41_0 s0 = mk41_0(acc);
    bump41_0(&s0, 1);
    acc += probe41_0(&s0);
    acc += read41_0(&s0);
    acc += classify41_0(1, acc, acc);
    acc += accum41_0(3);
    acc += guard41_0(acc);
    S41_1 s1 = mk41_1(acc);
    bump41_1(&s1, 2);
    acc += probe41_1(&s1);
    acc += read41_1(&s1);
    acc += classify41_1(1, acc, acc);
    acc += accum41_1(9);
    acc += guard41_1(acc);
    S41_2 s2 = mk41_2(acc);
    bump41_2(&s2, 7);
    acc += probe41_2(&s2);
    acc += read41_2(&s2);
    acc += classify41_2(1, acc, acc);
    acc += accum41_2(6);
    acc += guard41_2(acc);
    S41_3 s3 = mk41_3(acc);
    bump41_3(&s3, 9);
    acc += probe41_3(&s3);
    acc += read41_3(&s3);
    acc += classify41_3(1, acc, acc);
    acc += accum41_3(4);
    acc += guard41_3(acc);
    return clampi(acc);
}
