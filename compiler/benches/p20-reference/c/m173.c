/* GENERATED C mirror of reference module m173. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S173_0;

static S173_0 mk173_0(long a) {
    S173_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe173_0(const S173_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read173_0(const S173_0 *s) {
    return s->a * 7;
}
static void bump173_0(S173_0 *s, long d) {
    s->a = s->a + d;
}
static long classify173_0(int tag, long a, long b) {
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
static long accum173_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard173_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S173_1;

static S173_1 mk173_1(long a) {
    S173_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe173_1(const S173_1 *s) {
    return s->a + s->n0;
}
static long read173_1(const S173_1 *s) {
    return s->a * 7;
}
static void bump173_1(S173_1 *s, long d) {
    s->a = s->a + d;
}
static long classify173_1(int tag, long a, long b) {
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
static long accum173_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard173_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S173_2;

static S173_2 mk173_2(long a) {
    S173_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe173_2(const S173_2 *s) {
    return s->a + s->n0;
}
static long read173_2(const S173_2 *s) {
    return s->a * 3;
}
static void bump173_2(S173_2 *s, long d) {
    s->a = s->a + d;
}
static long classify173_2(int tag, long a, long b) {
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
static long accum173_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard173_2(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S173_3;

static S173_3 mk173_3(long a) {
    S173_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe173_3(const S173_3 *s) {
    return s->a + s->n0;
}
static long read173_3(const S173_3 *s) {
    return s->a * 2;
}
static void bump173_3(S173_3 *s, long d) {
    s->a = s->a + d;
}
static long classify173_3(int tag, long a, long b) {
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
static long accum173_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard173_3(long x) {
    return x + 9;
}

long f173(long x) {
    long acc = x;
    acc += f090(x + 1);
    acc += f112(x + 2);
    S173_0 s0 = mk173_0(acc);
    bump173_0(&s0, 6);
    acc += probe173_0(&s0);
    acc += read173_0(&s0);
    acc += classify173_0(1, acc, acc);
    acc += accum173_0(3);
    acc += guard173_0(acc);
    S173_1 s1 = mk173_1(acc);
    bump173_1(&s1, 7);
    acc += probe173_1(&s1);
    acc += read173_1(&s1);
    acc += classify173_1(1, acc, acc);
    acc += accum173_1(7);
    acc += guard173_1(acc);
    S173_2 s2 = mk173_2(acc);
    bump173_2(&s2, 4);
    acc += probe173_2(&s2);
    acc += read173_2(&s2);
    acc += classify173_2(1, acc, acc);
    acc += accum173_2(3);
    acc += guard173_2(acc);
    S173_3 s3 = mk173_3(acc);
    bump173_3(&s3, 2);
    acc += probe173_3(&s3);
    acc += read173_3(&s3);
    acc += classify173_3(1, acc, acc);
    acc += accum173_3(6);
    acc += guard173_3(acc);
    return clampi(acc);
}
