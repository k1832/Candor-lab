/* GENERATED C mirror of reference module m053. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S53_0;

static S53_0 mk53_0(long a) {
    S53_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe53_0(const S53_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read53_0(const S53_0 *s) {
    return s->a * 7;
}
static void bump53_0(S53_0 *s, long d) {
    s->a = s->a + d;
}
static long classify53_0(int tag, long a, long b) {
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
static long accum53_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard53_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S53_1;

static S53_1 mk53_1(long a) {
    S53_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe53_1(const S53_1 *s) {
    return s->a + s->n0;
}
static long read53_1(const S53_1 *s) {
    return s->a * 2;
}
static void bump53_1(S53_1 *s, long d) {
    s->a = s->a + d;
}
static long classify53_1(int tag, long a, long b) {
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
static long accum53_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard53_1(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S53_2;

static S53_2 mk53_2(long a) {
    S53_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe53_2(const S53_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read53_2(const S53_2 *s) {
    return s->a * 4;
}
static void bump53_2(S53_2 *s, long d) {
    s->a = s->a + d;
}
static long classify53_2(int tag, long a, long b) {
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
static long accum53_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard53_2(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S53_3;

static S53_3 mk53_3(long a) {
    S53_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe53_3(const S53_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read53_3(const S53_3 *s) {
    return s->a * 7;
}
static void bump53_3(S53_3 *s, long d) {
    s->a = s->a + d;
}
static long classify53_3(int tag, long a, long b) {
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
static long accum53_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard53_3(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S53_4;

static S53_4 mk53_4(long a) {
    S53_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe53_4(const S53_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read53_4(const S53_4 *s) {
    return s->a * 5;
}
static void bump53_4(S53_4 *s, long d) {
    s->a = s->a + d;
}
static long classify53_4(int tag, long a, long b) {
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
static long accum53_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard53_4(long x) {
    return x + 7;
}

long f053(long x) {
    long acc = x;
    acc += f039(x + 1);
    acc += f044(x + 2);
    S53_0 s0 = mk53_0(acc);
    bump53_0(&s0, 8);
    acc += probe53_0(&s0);
    acc += read53_0(&s0);
    acc += classify53_0(1, acc, acc);
    acc += accum53_0(8);
    acc += guard53_0(acc);
    S53_1 s1 = mk53_1(acc);
    bump53_1(&s1, 7);
    acc += probe53_1(&s1);
    acc += read53_1(&s1);
    acc += classify53_1(1, acc, acc);
    acc += accum53_1(9);
    acc += guard53_1(acc);
    S53_2 s2 = mk53_2(acc);
    bump53_2(&s2, 1);
    acc += probe53_2(&s2);
    acc += read53_2(&s2);
    acc += classify53_2(1, acc, acc);
    acc += accum53_2(4);
    acc += guard53_2(acc);
    S53_3 s3 = mk53_3(acc);
    bump53_3(&s3, 9);
    acc += probe53_3(&s3);
    acc += read53_3(&s3);
    acc += classify53_3(1, acc, acc);
    acc += accum53_3(9);
    acc += guard53_3(acc);
    S53_4 s4 = mk53_4(acc);
    bump53_4(&s4, 2);
    acc += probe53_4(&s4);
    acc += read53_4(&s4);
    acc += classify53_4(1, acc, acc);
    acc += accum53_4(9);
    acc += guard53_4(acc);
    return clampi(acc);
}
