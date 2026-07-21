/* GENERATED C mirror of reference module m155. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S155_0;

static S155_0 mk155_0(long a) {
    S155_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe155_0(const S155_0 *s) {
    return s->a + s->n0;
}
static long read155_0(const S155_0 *s) {
    return s->a * 4;
}
static void bump155_0(S155_0 *s, long d) {
    s->a = s->a + d;
}
static long classify155_0(int tag, long a, long b) {
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
static long accum155_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard155_0(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S155_1;

static S155_1 mk155_1(long a) {
    S155_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe155_1(const S155_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read155_1(const S155_1 *s) {
    return s->a * 3;
}
static void bump155_1(S155_1 *s, long d) {
    s->a = s->a + d;
}
static long classify155_1(int tag, long a, long b) {
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
static long accum155_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard155_1(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S155_2;

static S155_2 mk155_2(long a) {
    S155_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe155_2(const S155_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read155_2(const S155_2 *s) {
    return s->a * 3;
}
static void bump155_2(S155_2 *s, long d) {
    s->a = s->a + d;
}
static long classify155_2(int tag, long a, long b) {
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
static long accum155_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard155_2(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S155_3;

static S155_3 mk155_3(long a) {
    S155_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe155_3(const S155_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read155_3(const S155_3 *s) {
    return s->a * 7;
}
static void bump155_3(S155_3 *s, long d) {
    s->a = s->a + d;
}
static long classify155_3(int tag, long a, long b) {
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
static long accum155_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard155_3(long x) {
    return x + 5;
}

long f155(long x) {
    long acc = x;
    acc += f031(x + 1);
    acc += f101(x + 2);
    S155_0 s0 = mk155_0(acc);
    bump155_0(&s0, 9);
    acc += probe155_0(&s0);
    acc += read155_0(&s0);
    acc += classify155_0(1, acc, acc);
    acc += accum155_0(8);
    acc += guard155_0(acc);
    S155_1 s1 = mk155_1(acc);
    bump155_1(&s1, 6);
    acc += probe155_1(&s1);
    acc += read155_1(&s1);
    acc += classify155_1(1, acc, acc);
    acc += accum155_1(9);
    acc += guard155_1(acc);
    S155_2 s2 = mk155_2(acc);
    bump155_2(&s2, 9);
    acc += probe155_2(&s2);
    acc += read155_2(&s2);
    acc += classify155_2(1, acc, acc);
    acc += accum155_2(4);
    acc += guard155_2(acc);
    S155_3 s3 = mk155_3(acc);
    bump155_3(&s3, 8);
    acc += probe155_3(&s3);
    acc += read155_3(&s3);
    acc += classify155_3(1, acc, acc);
    acc += accum155_3(3);
    acc += guard155_3(acc);
    return clampi(acc);
}
