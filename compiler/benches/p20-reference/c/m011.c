/* GENERATED C mirror of reference module m011. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S11_0;

static S11_0 mk11_0(long a) {
    S11_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe11_0(const S11_0 *s) {
    return s->a + s->n0;
}
static long read11_0(const S11_0 *s) {
    return s->a * 5;
}
static void bump11_0(S11_0 *s, long d) {
    s->a = s->a + d;
}
static long classify11_0(int tag, long a, long b) {
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
static long accum11_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard11_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S11_1;

static S11_1 mk11_1(long a) {
    S11_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe11_1(const S11_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read11_1(const S11_1 *s) {
    return s->a * 3;
}
static void bump11_1(S11_1 *s, long d) {
    s->a = s->a + d;
}
static long classify11_1(int tag, long a, long b) {
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
static long accum11_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard11_1(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S11_2;

static S11_2 mk11_2(long a) {
    S11_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe11_2(const S11_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read11_2(const S11_2 *s) {
    return s->a * 6;
}
static void bump11_2(S11_2 *s, long d) {
    s->a = s->a + d;
}
static long classify11_2(int tag, long a, long b) {
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
static long accum11_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard11_2(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S11_3;

static S11_3 mk11_3(long a) {
    S11_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe11_3(const S11_3 *s) {
    return s->a + s->n0;
}
static long read11_3(const S11_3 *s) {
    return s->a * 6;
}
static void bump11_3(S11_3 *s, long d) {
    s->a = s->a + d;
}
static long classify11_3(int tag, long a, long b) {
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
static long accum11_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard11_3(long x) {
    return x + 7;
}

long f011(long x) {
    long acc = x;
    acc += f005(x + 1);
    S11_0 s0 = mk11_0(acc);
    bump11_0(&s0, 4);
    acc += probe11_0(&s0);
    acc += read11_0(&s0);
    acc += classify11_0(1, acc, acc);
    acc += accum11_0(6);
    acc += guard11_0(acc);
    S11_1 s1 = mk11_1(acc);
    bump11_1(&s1, 7);
    acc += probe11_1(&s1);
    acc += read11_1(&s1);
    acc += classify11_1(1, acc, acc);
    acc += accum11_1(8);
    acc += guard11_1(acc);
    S11_2 s2 = mk11_2(acc);
    bump11_2(&s2, 7);
    acc += probe11_2(&s2);
    acc += read11_2(&s2);
    acc += classify11_2(1, acc, acc);
    acc += accum11_2(6);
    acc += guard11_2(acc);
    S11_3 s3 = mk11_3(acc);
    bump11_3(&s3, 9);
    acc += probe11_3(&s3);
    acc += read11_3(&s3);
    acc += classify11_3(1, acc, acc);
    acc += accum11_3(8);
    acc += guard11_3(acc);
    return clampi(acc);
}
