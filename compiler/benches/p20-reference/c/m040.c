/* GENERATED C mirror of reference module m040. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S40_0;

static S40_0 mk40_0(long a) {
    S40_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe40_0(const S40_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read40_0(const S40_0 *s) {
    return s->a * 2;
}
static void bump40_0(S40_0 *s, long d) {
    s->a = s->a + d;
}
static long classify40_0(int tag, long a, long b) {
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
static long accum40_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard40_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S40_1;

static S40_1 mk40_1(long a) {
    S40_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe40_1(const S40_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read40_1(const S40_1 *s) {
    return s->a * 2;
}
static void bump40_1(S40_1 *s, long d) {
    s->a = s->a + d;
}
static long classify40_1(int tag, long a, long b) {
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
static long accum40_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard40_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S40_2;

static S40_2 mk40_2(long a) {
    S40_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe40_2(const S40_2 *s) {
    return s->a + s->n0;
}
static long read40_2(const S40_2 *s) {
    return s->a * 5;
}
static void bump40_2(S40_2 *s, long d) {
    s->a = s->a + d;
}
static long classify40_2(int tag, long a, long b) {
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
static long accum40_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard40_2(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S40_3;

static S40_3 mk40_3(long a) {
    S40_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe40_3(const S40_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read40_3(const S40_3 *s) {
    return s->a * 2;
}
static void bump40_3(S40_3 *s, long d) {
    s->a = s->a + d;
}
static long classify40_3(int tag, long a, long b) {
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
static long accum40_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard40_3(long x) {
    return x + 7;
}

long f040(long x) {
    long acc = x;
    acc += f006(x + 1);
    acc += f009(x + 2);
    acc += f013(x + 3);
    S40_0 s0 = mk40_0(acc);
    bump40_0(&s0, 1);
    acc += probe40_0(&s0);
    acc += read40_0(&s0);
    acc += classify40_0(1, acc, acc);
    acc += accum40_0(3);
    acc += guard40_0(acc);
    S40_1 s1 = mk40_1(acc);
    bump40_1(&s1, 9);
    acc += probe40_1(&s1);
    acc += read40_1(&s1);
    acc += classify40_1(1, acc, acc);
    acc += accum40_1(4);
    acc += guard40_1(acc);
    S40_2 s2 = mk40_2(acc);
    bump40_2(&s2, 7);
    acc += probe40_2(&s2);
    acc += read40_2(&s2);
    acc += classify40_2(1, acc, acc);
    acc += accum40_2(4);
    acc += guard40_2(acc);
    S40_3 s3 = mk40_3(acc);
    bump40_3(&s3, 9);
    acc += probe40_3(&s3);
    acc += read40_3(&s3);
    acc += classify40_3(1, acc, acc);
    acc += accum40_3(9);
    acc += guard40_3(acc);
    return clampi(acc);
}
