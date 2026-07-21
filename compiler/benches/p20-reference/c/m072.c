/* GENERATED C mirror of reference module m072. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S72_0;

static S72_0 mk72_0(long a) {
    S72_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe72_0(const S72_0 *s) {
    return s->a + s->n0;
}
static long read72_0(const S72_0 *s) {
    return s->a * 5;
}
static void bump72_0(S72_0 *s, long d) {
    s->a = s->a + d;
}
static long classify72_0(int tag, long a, long b) {
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
static long accum72_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard72_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S72_1;

static S72_1 mk72_1(long a) {
    S72_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe72_1(const S72_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read72_1(const S72_1 *s) {
    return s->a * 6;
}
static void bump72_1(S72_1 *s, long d) {
    s->a = s->a + d;
}
static long classify72_1(int tag, long a, long b) {
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
static long accum72_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard72_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S72_2;

static S72_2 mk72_2(long a) {
    S72_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe72_2(const S72_2 *s) {
    return s->a + s->n0;
}
static long read72_2(const S72_2 *s) {
    return s->a * 4;
}
static void bump72_2(S72_2 *s, long d) {
    s->a = s->a + d;
}
static long classify72_2(int tag, long a, long b) {
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
static long accum72_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard72_2(long x) {
    return x + 5;
}

long f072(long x) {
    long acc = x;
    acc += f013(x + 1);
    acc += f017(x + 2);
    acc += f021(x + 3);
    S72_0 s0 = mk72_0(acc);
    bump72_0(&s0, 7);
    acc += probe72_0(&s0);
    acc += read72_0(&s0);
    acc += classify72_0(1, acc, acc);
    acc += accum72_0(4);
    acc += guard72_0(acc);
    S72_1 s1 = mk72_1(acc);
    bump72_1(&s1, 1);
    acc += probe72_1(&s1);
    acc += read72_1(&s1);
    acc += classify72_1(1, acc, acc);
    acc += accum72_1(5);
    acc += guard72_1(acc);
    S72_2 s2 = mk72_2(acc);
    bump72_2(&s2, 5);
    acc += probe72_2(&s2);
    acc += read72_2(&s2);
    acc += classify72_2(1, acc, acc);
    acc += accum72_2(9);
    acc += guard72_2(acc);
    return clampi(acc);
}
