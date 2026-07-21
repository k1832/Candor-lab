/* GENERATED C mirror of reference module m143. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S143_0;

static S143_0 mk143_0(long a) {
    S143_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe143_0(const S143_0 *s) {
    return s->a + s->n0;
}
static long read143_0(const S143_0 *s) {
    return s->a * 7;
}
static void bump143_0(S143_0 *s, long d) {
    s->a = s->a + d;
}
static long classify143_0(int tag, long a, long b) {
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
static long accum143_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard143_0(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S143_1;

static S143_1 mk143_1(long a) {
    S143_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe143_1(const S143_1 *s) {
    return s->a + s->n0;
}
static long read143_1(const S143_1 *s) {
    return s->a * 2;
}
static void bump143_1(S143_1 *s, long d) {
    s->a = s->a + d;
}
static long classify143_1(int tag, long a, long b) {
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
static long accum143_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard143_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S143_2;

static S143_2 mk143_2(long a) {
    S143_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe143_2(const S143_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read143_2(const S143_2 *s) {
    return s->a * 2;
}
static void bump143_2(S143_2 *s, long d) {
    s->a = s->a + d;
}
static long classify143_2(int tag, long a, long b) {
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
static long accum143_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard143_2(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S143_3;

static S143_3 mk143_3(long a) {
    S143_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe143_3(const S143_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read143_3(const S143_3 *s) {
    return s->a * 3;
}
static void bump143_3(S143_3 *s, long d) {
    s->a = s->a + d;
}
static long classify143_3(int tag, long a, long b) {
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
static long accum143_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard143_3(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S143_4;

static S143_4 mk143_4(long a) {
    S143_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe143_4(const S143_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read143_4(const S143_4 *s) {
    return s->a * 2;
}
static void bump143_4(S143_4 *s, long d) {
    s->a = s->a + d;
}
static long classify143_4(int tag, long a, long b) {
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
static long accum143_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard143_4(long x) {
    return x + 1;
}

long f143(long x) {
    long acc = x;
    acc += f017(x + 1);
    acc += f075(x + 2);
    acc += f128(x + 3);
    acc += f136(x + 4);
    S143_0 s0 = mk143_0(acc);
    bump143_0(&s0, 4);
    acc += probe143_0(&s0);
    acc += read143_0(&s0);
    acc += classify143_0(1, acc, acc);
    acc += accum143_0(6);
    acc += guard143_0(acc);
    S143_1 s1 = mk143_1(acc);
    bump143_1(&s1, 7);
    acc += probe143_1(&s1);
    acc += read143_1(&s1);
    acc += classify143_1(1, acc, acc);
    acc += accum143_1(7);
    acc += guard143_1(acc);
    S143_2 s2 = mk143_2(acc);
    bump143_2(&s2, 6);
    acc += probe143_2(&s2);
    acc += read143_2(&s2);
    acc += classify143_2(1, acc, acc);
    acc += accum143_2(3);
    acc += guard143_2(acc);
    S143_3 s3 = mk143_3(acc);
    bump143_3(&s3, 9);
    acc += probe143_3(&s3);
    acc += read143_3(&s3);
    acc += classify143_3(1, acc, acc);
    acc += accum143_3(6);
    acc += guard143_3(acc);
    S143_4 s4 = mk143_4(acc);
    bump143_4(&s4, 9);
    acc += probe143_4(&s4);
    acc += read143_4(&s4);
    acc += classify143_4(1, acc, acc);
    acc += accum143_4(9);
    acc += guard143_4(acc);
    return clampi(acc);
}
