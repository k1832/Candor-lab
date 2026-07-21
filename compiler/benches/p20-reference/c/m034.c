/* GENERATED C mirror of reference module m034. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S34_0;

static S34_0 mk34_0(long a) {
    S34_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe34_0(const S34_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read34_0(const S34_0 *s) {
    return s->a * 3;
}
static void bump34_0(S34_0 *s, long d) {
    s->a = s->a + d;
}
static long classify34_0(int tag, long a, long b) {
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
static long accum34_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard34_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S34_1;

static S34_1 mk34_1(long a) {
    S34_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe34_1(const S34_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read34_1(const S34_1 *s) {
    return s->a * 7;
}
static void bump34_1(S34_1 *s, long d) {
    s->a = s->a + d;
}
static long classify34_1(int tag, long a, long b) {
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
static long accum34_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard34_1(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S34_2;

static S34_2 mk34_2(long a) {
    S34_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe34_2(const S34_2 *s) {
    return s->a + s->n0;
}
static long read34_2(const S34_2 *s) {
    return s->a * 3;
}
static void bump34_2(S34_2 *s, long d) {
    s->a = s->a + d;
}
static long classify34_2(int tag, long a, long b) {
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
static long accum34_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard34_2(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S34_3;

static S34_3 mk34_3(long a) {
    S34_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe34_3(const S34_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read34_3(const S34_3 *s) {
    return s->a * 3;
}
static void bump34_3(S34_3 *s, long d) {
    s->a = s->a + d;
}
static long classify34_3(int tag, long a, long b) {
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
static long accum34_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard34_3(long x) {
    return x + 9;
}

long f034(long x) {
    long acc = x;
    acc += f011(x + 1);
    S34_0 s0 = mk34_0(acc);
    bump34_0(&s0, 4);
    acc += probe34_0(&s0);
    acc += read34_0(&s0);
    acc += classify34_0(1, acc, acc);
    acc += accum34_0(4);
    acc += guard34_0(acc);
    S34_1 s1 = mk34_1(acc);
    bump34_1(&s1, 1);
    acc += probe34_1(&s1);
    acc += read34_1(&s1);
    acc += classify34_1(1, acc, acc);
    acc += accum34_1(7);
    acc += guard34_1(acc);
    S34_2 s2 = mk34_2(acc);
    bump34_2(&s2, 4);
    acc += probe34_2(&s2);
    acc += read34_2(&s2);
    acc += classify34_2(1, acc, acc);
    acc += accum34_2(3);
    acc += guard34_2(acc);
    S34_3 s3 = mk34_3(acc);
    bump34_3(&s3, 2);
    acc += probe34_3(&s3);
    acc += read34_3(&s3);
    acc += classify34_3(1, acc, acc);
    acc += accum34_3(7);
    acc += guard34_3(acc);
    return clampi(acc);
}
