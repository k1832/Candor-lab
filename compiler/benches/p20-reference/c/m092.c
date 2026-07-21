/* GENERATED C mirror of reference module m092. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S92_0;

static S92_0 mk92_0(long a) {
    S92_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe92_0(const S92_0 *s) {
    return s->a + s->n0;
}
static long read92_0(const S92_0 *s) {
    return s->a * 4;
}
static void bump92_0(S92_0 *s, long d) {
    s->a = s->a + d;
}
static long classify92_0(int tag, long a, long b) {
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
static long accum92_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard92_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S92_1;

static S92_1 mk92_1(long a) {
    S92_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe92_1(const S92_1 *s) {
    return s->a + s->n0;
}
static long read92_1(const S92_1 *s) {
    return s->a * 2;
}
static void bump92_1(S92_1 *s, long d) {
    s->a = s->a + d;
}
static long classify92_1(int tag, long a, long b) {
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
static long accum92_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard92_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S92_2;

static S92_2 mk92_2(long a) {
    S92_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe92_2(const S92_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read92_2(const S92_2 *s) {
    return s->a * 6;
}
static void bump92_2(S92_2 *s, long d) {
    s->a = s->a + d;
}
static long classify92_2(int tag, long a, long b) {
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
static long accum92_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard92_2(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S92_3;

static S92_3 mk92_3(long a) {
    S92_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe92_3(const S92_3 *s) {
    return s->a + s->n0;
}
static long read92_3(const S92_3 *s) {
    return s->a * 6;
}
static void bump92_3(S92_3 *s, long d) {
    s->a = s->a + d;
}
static long classify92_3(int tag, long a, long b) {
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
static long accum92_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard92_3(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S92_4;

static S92_4 mk92_4(long a) {
    S92_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe92_4(const S92_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read92_4(const S92_4 *s) {
    return s->a * 7;
}
static void bump92_4(S92_4 *s, long d) {
    s->a = s->a + d;
}
static long classify92_4(int tag, long a, long b) {
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
static long accum92_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard92_4(long x) {
    return x + 4;
}

long f092(long x) {
    long acc = x;
    acc += f006(x + 1);
    acc += f031(x + 2);
    acc += f071(x + 3);
    acc += f077(x + 4);
    S92_0 s0 = mk92_0(acc);
    bump92_0(&s0, 8);
    acc += probe92_0(&s0);
    acc += read92_0(&s0);
    acc += classify92_0(1, acc, acc);
    acc += accum92_0(9);
    acc += guard92_0(acc);
    S92_1 s1 = mk92_1(acc);
    bump92_1(&s1, 8);
    acc += probe92_1(&s1);
    acc += read92_1(&s1);
    acc += classify92_1(1, acc, acc);
    acc += accum92_1(3);
    acc += guard92_1(acc);
    S92_2 s2 = mk92_2(acc);
    bump92_2(&s2, 8);
    acc += probe92_2(&s2);
    acc += read92_2(&s2);
    acc += classify92_2(1, acc, acc);
    acc += accum92_2(4);
    acc += guard92_2(acc);
    S92_3 s3 = mk92_3(acc);
    bump92_3(&s3, 9);
    acc += probe92_3(&s3);
    acc += read92_3(&s3);
    acc += classify92_3(1, acc, acc);
    acc += accum92_3(3);
    acc += guard92_3(acc);
    S92_4 s4 = mk92_4(acc);
    bump92_4(&s4, 5);
    acc += probe92_4(&s4);
    acc += read92_4(&s4);
    acc += classify92_4(1, acc, acc);
    acc += accum92_4(4);
    acc += guard92_4(acc);
    return clampi(acc);
}
