/* GENERATED C mirror of reference module m131. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S131_0;

static S131_0 mk131_0(long a) {
    S131_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe131_0(const S131_0 *s) {
    return s->a + s->n0;
}
static long read131_0(const S131_0 *s) {
    return s->a * 7;
}
static void bump131_0(S131_0 *s, long d) {
    s->a = s->a + d;
}
static long classify131_0(int tag, long a, long b) {
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
static long accum131_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard131_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S131_1;

static S131_1 mk131_1(long a) {
    S131_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe131_1(const S131_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read131_1(const S131_1 *s) {
    return s->a * 4;
}
static void bump131_1(S131_1 *s, long d) {
    s->a = s->a + d;
}
static long classify131_1(int tag, long a, long b) {
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
static long accum131_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard131_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S131_2;

static S131_2 mk131_2(long a) {
    S131_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe131_2(const S131_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read131_2(const S131_2 *s) {
    return s->a * 7;
}
static void bump131_2(S131_2 *s, long d) {
    s->a = s->a + d;
}
static long classify131_2(int tag, long a, long b) {
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
static long accum131_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard131_2(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S131_3;

static S131_3 mk131_3(long a) {
    S131_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe131_3(const S131_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read131_3(const S131_3 *s) {
    return s->a * 7;
}
static void bump131_3(S131_3 *s, long d) {
    s->a = s->a + d;
}
static long classify131_3(int tag, long a, long b) {
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
static long accum131_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard131_3(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S131_4;

static S131_4 mk131_4(long a) {
    S131_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe131_4(const S131_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read131_4(const S131_4 *s) {
    return s->a * 6;
}
static void bump131_4(S131_4 *s, long d) {
    s->a = s->a + d;
}
static long classify131_4(int tag, long a, long b) {
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
static long accum131_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard131_4(long x) {
    return x + 3;
}

long f131(long x) {
    long acc = x;
    acc += f019(x + 1);
    acc += f028(x + 2);
    acc += f061(x + 3);
    acc += f102(x + 4);
    S131_0 s0 = mk131_0(acc);
    bump131_0(&s0, 9);
    acc += probe131_0(&s0);
    acc += read131_0(&s0);
    acc += classify131_0(1, acc, acc);
    acc += accum131_0(9);
    acc += guard131_0(acc);
    S131_1 s1 = mk131_1(acc);
    bump131_1(&s1, 4);
    acc += probe131_1(&s1);
    acc += read131_1(&s1);
    acc += classify131_1(1, acc, acc);
    acc += accum131_1(8);
    acc += guard131_1(acc);
    S131_2 s2 = mk131_2(acc);
    bump131_2(&s2, 6);
    acc += probe131_2(&s2);
    acc += read131_2(&s2);
    acc += classify131_2(1, acc, acc);
    acc += accum131_2(8);
    acc += guard131_2(acc);
    S131_3 s3 = mk131_3(acc);
    bump131_3(&s3, 2);
    acc += probe131_3(&s3);
    acc += read131_3(&s3);
    acc += classify131_3(1, acc, acc);
    acc += accum131_3(6);
    acc += guard131_3(acc);
    S131_4 s4 = mk131_4(acc);
    bump131_4(&s4, 3);
    acc += probe131_4(&s4);
    acc += read131_4(&s4);
    acc += classify131_4(1, acc, acc);
    acc += accum131_4(8);
    acc += guard131_4(acc);
    return clampi(acc);
}
