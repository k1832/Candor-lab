/* GENERATED C mirror of reference module m156. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S156_0;

static S156_0 mk156_0(long a) {
    S156_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe156_0(const S156_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read156_0(const S156_0 *s) {
    return s->a * 2;
}
static void bump156_0(S156_0 *s, long d) {
    s->a = s->a + d;
}
static long classify156_0(int tag, long a, long b) {
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
static long accum156_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard156_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S156_1;

static S156_1 mk156_1(long a) {
    S156_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe156_1(const S156_1 *s) {
    return s->a + s->n0;
}
static long read156_1(const S156_1 *s) {
    return s->a * 6;
}
static void bump156_1(S156_1 *s, long d) {
    s->a = s->a + d;
}
static long classify156_1(int tag, long a, long b) {
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
static long accum156_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard156_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S156_2;

static S156_2 mk156_2(long a) {
    S156_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe156_2(const S156_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read156_2(const S156_2 *s) {
    return s->a * 3;
}
static void bump156_2(S156_2 *s, long d) {
    s->a = s->a + d;
}
static long classify156_2(int tag, long a, long b) {
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
static long accum156_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard156_2(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S156_3;

static S156_3 mk156_3(long a) {
    S156_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe156_3(const S156_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read156_3(const S156_3 *s) {
    return s->a * 4;
}
static void bump156_3(S156_3 *s, long d) {
    s->a = s->a + d;
}
static long classify156_3(int tag, long a, long b) {
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
static long accum156_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard156_3(long x) {
    return x + 9;
}

long f156(long x) {
    long acc = x;
    acc += f008(x + 1);
    acc += f045(x + 2);
    acc += f070(x + 3);
    S156_0 s0 = mk156_0(acc);
    bump156_0(&s0, 7);
    acc += probe156_0(&s0);
    acc += read156_0(&s0);
    acc += classify156_0(1, acc, acc);
    acc += accum156_0(7);
    acc += guard156_0(acc);
    S156_1 s1 = mk156_1(acc);
    bump156_1(&s1, 9);
    acc += probe156_1(&s1);
    acc += read156_1(&s1);
    acc += classify156_1(1, acc, acc);
    acc += accum156_1(9);
    acc += guard156_1(acc);
    S156_2 s2 = mk156_2(acc);
    bump156_2(&s2, 3);
    acc += probe156_2(&s2);
    acc += read156_2(&s2);
    acc += classify156_2(1, acc, acc);
    acc += accum156_2(4);
    acc += guard156_2(acc);
    S156_3 s3 = mk156_3(acc);
    bump156_3(&s3, 7);
    acc += probe156_3(&s3);
    acc += read156_3(&s3);
    acc += classify156_3(1, acc, acc);
    acc += accum156_3(9);
    acc += guard156_3(acc);
    return clampi(acc);
}
