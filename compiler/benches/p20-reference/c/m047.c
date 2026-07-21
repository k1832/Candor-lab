/* GENERATED C mirror of reference module m047. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S47_0;

static S47_0 mk47_0(long a) {
    S47_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe47_0(const S47_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read47_0(const S47_0 *s) {
    return s->a * 4;
}
static void bump47_0(S47_0 *s, long d) {
    s->a = s->a + d;
}
static long classify47_0(int tag, long a, long b) {
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
static long accum47_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard47_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S47_1;

static S47_1 mk47_1(long a) {
    S47_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe47_1(const S47_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read47_1(const S47_1 *s) {
    return s->a * 2;
}
static void bump47_1(S47_1 *s, long d) {
    s->a = s->a + d;
}
static long classify47_1(int tag, long a, long b) {
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
static long accum47_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard47_1(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S47_2;

static S47_2 mk47_2(long a) {
    S47_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe47_2(const S47_2 *s) {
    return s->a + s->n0;
}
static long read47_2(const S47_2 *s) {
    return s->a * 2;
}
static void bump47_2(S47_2 *s, long d) {
    s->a = s->a + d;
}
static long classify47_2(int tag, long a, long b) {
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
static long accum47_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard47_2(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S47_3;

static S47_3 mk47_3(long a) {
    S47_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe47_3(const S47_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read47_3(const S47_3 *s) {
    return s->a * 6;
}
static void bump47_3(S47_3 *s, long d) {
    s->a = s->a + d;
}
static long classify47_3(int tag, long a, long b) {
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
static long accum47_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard47_3(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S47_4;

static S47_4 mk47_4(long a) {
    S47_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe47_4(const S47_4 *s) {
    return s->a + s->n0;
}
static long read47_4(const S47_4 *s) {
    return s->a * 2;
}
static void bump47_4(S47_4 *s, long d) {
    s->a = s->a + d;
}
static long classify47_4(int tag, long a, long b) {
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
static long accum47_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard47_4(long x) {
    return x + 7;
}

long f047(long x) {
    long acc = x;
    acc += f002(x + 1);
    acc += f014(x + 2);
    S47_0 s0 = mk47_0(acc);
    bump47_0(&s0, 9);
    acc += probe47_0(&s0);
    acc += read47_0(&s0);
    acc += classify47_0(1, acc, acc);
    acc += accum47_0(3);
    acc += guard47_0(acc);
    S47_1 s1 = mk47_1(acc);
    bump47_1(&s1, 5);
    acc += probe47_1(&s1);
    acc += read47_1(&s1);
    acc += classify47_1(1, acc, acc);
    acc += accum47_1(6);
    acc += guard47_1(acc);
    S47_2 s2 = mk47_2(acc);
    bump47_2(&s2, 9);
    acc += probe47_2(&s2);
    acc += read47_2(&s2);
    acc += classify47_2(1, acc, acc);
    acc += accum47_2(7);
    acc += guard47_2(acc);
    S47_3 s3 = mk47_3(acc);
    bump47_3(&s3, 3);
    acc += probe47_3(&s3);
    acc += read47_3(&s3);
    acc += classify47_3(1, acc, acc);
    acc += accum47_3(8);
    acc += guard47_3(acc);
    S47_4 s4 = mk47_4(acc);
    bump47_4(&s4, 2);
    acc += probe47_4(&s4);
    acc += read47_4(&s4);
    acc += classify47_4(1, acc, acc);
    acc += accum47_4(3);
    acc += guard47_4(acc);
    return clampi(acc);
}
