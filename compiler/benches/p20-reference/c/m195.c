/* GENERATED C mirror of reference module m195. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S195_0;

static S195_0 mk195_0(long a) {
    S195_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe195_0(const S195_0 *s) {
    return s->a + s->n0;
}
static long read195_0(const S195_0 *s) {
    return s->a * 6;
}
static void bump195_0(S195_0 *s, long d) {
    s->a = s->a + d;
}
static long classify195_0(int tag, long a, long b) {
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
static long accum195_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard195_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S195_1;

static S195_1 mk195_1(long a) {
    S195_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe195_1(const S195_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read195_1(const S195_1 *s) {
    return s->a * 7;
}
static void bump195_1(S195_1 *s, long d) {
    s->a = s->a + d;
}
static long classify195_1(int tag, long a, long b) {
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
static long accum195_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard195_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S195_2;

static S195_2 mk195_2(long a) {
    S195_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe195_2(const S195_2 *s) {
    return s->a + s->n0;
}
static long read195_2(const S195_2 *s) {
    return s->a * 7;
}
static void bump195_2(S195_2 *s, long d) {
    s->a = s->a + d;
}
static long classify195_2(int tag, long a, long b) {
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
static long accum195_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard195_2(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S195_3;

static S195_3 mk195_3(long a) {
    S195_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe195_3(const S195_3 *s) {
    return s->a + s->n0;
}
static long read195_3(const S195_3 *s) {
    return s->a * 2;
}
static void bump195_3(S195_3 *s, long d) {
    s->a = s->a + d;
}
static long classify195_3(int tag, long a, long b) {
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
static long accum195_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard195_3(long x) {
    return x + 5;
}

long f195(long x) {
    long acc = x;
    acc += f052(x + 1);
    acc += f111(x + 2);
    S195_0 s0 = mk195_0(acc);
    bump195_0(&s0, 5);
    acc += probe195_0(&s0);
    acc += read195_0(&s0);
    acc += classify195_0(1, acc, acc);
    acc += accum195_0(8);
    acc += guard195_0(acc);
    S195_1 s1 = mk195_1(acc);
    bump195_1(&s1, 9);
    acc += probe195_1(&s1);
    acc += read195_1(&s1);
    acc += classify195_1(1, acc, acc);
    acc += accum195_1(7);
    acc += guard195_1(acc);
    S195_2 s2 = mk195_2(acc);
    bump195_2(&s2, 1);
    acc += probe195_2(&s2);
    acc += read195_2(&s2);
    acc += classify195_2(1, acc, acc);
    acc += accum195_2(8);
    acc += guard195_2(acc);
    S195_3 s3 = mk195_3(acc);
    bump195_3(&s3, 6);
    acc += probe195_3(&s3);
    acc += read195_3(&s3);
    acc += classify195_3(1, acc, acc);
    acc += accum195_3(9);
    acc += guard195_3(acc);
    return clampi(acc);
}
