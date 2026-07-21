/* GENERATED C mirror of reference module m172. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S172_0;

static S172_0 mk172_0(long a) {
    S172_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe172_0(const S172_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read172_0(const S172_0 *s) {
    return s->a * 2;
}
static void bump172_0(S172_0 *s, long d) {
    s->a = s->a + d;
}
static long classify172_0(int tag, long a, long b) {
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
static long accum172_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard172_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S172_1;

static S172_1 mk172_1(long a) {
    S172_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe172_1(const S172_1 *s) {
    return s->a + s->n0;
}
static long read172_1(const S172_1 *s) {
    return s->a * 4;
}
static void bump172_1(S172_1 *s, long d) {
    s->a = s->a + d;
}
static long classify172_1(int tag, long a, long b) {
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
static long accum172_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard172_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S172_2;

static S172_2 mk172_2(long a) {
    S172_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe172_2(const S172_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read172_2(const S172_2 *s) {
    return s->a * 7;
}
static void bump172_2(S172_2 *s, long d) {
    s->a = s->a + d;
}
static long classify172_2(int tag, long a, long b) {
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
static long accum172_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard172_2(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S172_3;

static S172_3 mk172_3(long a) {
    S172_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe172_3(const S172_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read172_3(const S172_3 *s) {
    return s->a * 7;
}
static void bump172_3(S172_3 *s, long d) {
    s->a = s->a + d;
}
static long classify172_3(int tag, long a, long b) {
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
static long accum172_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard172_3(long x) {
    return x + 1;
}

long f172(long x) {
    long acc = x;
    acc += f092(x + 1);
    acc += f141(x + 2);
    S172_0 s0 = mk172_0(acc);
    bump172_0(&s0, 3);
    acc += probe172_0(&s0);
    acc += read172_0(&s0);
    acc += classify172_0(1, acc, acc);
    acc += accum172_0(4);
    acc += guard172_0(acc);
    S172_1 s1 = mk172_1(acc);
    bump172_1(&s1, 5);
    acc += probe172_1(&s1);
    acc += read172_1(&s1);
    acc += classify172_1(1, acc, acc);
    acc += accum172_1(8);
    acc += guard172_1(acc);
    S172_2 s2 = mk172_2(acc);
    bump172_2(&s2, 3);
    acc += probe172_2(&s2);
    acc += read172_2(&s2);
    acc += classify172_2(1, acc, acc);
    acc += accum172_2(6);
    acc += guard172_2(acc);
    S172_3 s3 = mk172_3(acc);
    bump172_3(&s3, 6);
    acc += probe172_3(&s3);
    acc += read172_3(&s3);
    acc += classify172_3(1, acc, acc);
    acc += accum172_3(4);
    acc += guard172_3(acc);
    return clampi(acc);
}
