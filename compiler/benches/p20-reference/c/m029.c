/* GENERATED C mirror of reference module m029. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S29_0;

static S29_0 mk29_0(long a) {
    S29_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe29_0(const S29_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read29_0(const S29_0 *s) {
    return s->a * 4;
}
static void bump29_0(S29_0 *s, long d) {
    s->a = s->a + d;
}
static long classify29_0(int tag, long a, long b) {
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
static long accum29_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard29_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S29_1;

static S29_1 mk29_1(long a) {
    S29_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe29_1(const S29_1 *s) {
    return s->a + s->n0;
}
static long read29_1(const S29_1 *s) {
    return s->a * 7;
}
static void bump29_1(S29_1 *s, long d) {
    s->a = s->a + d;
}
static long classify29_1(int tag, long a, long b) {
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
static long accum29_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard29_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S29_2;

static S29_2 mk29_2(long a) {
    S29_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe29_2(const S29_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read29_2(const S29_2 *s) {
    return s->a * 7;
}
static void bump29_2(S29_2 *s, long d) {
    s->a = s->a + d;
}
static long classify29_2(int tag, long a, long b) {
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
static long accum29_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard29_2(long x) {
    return x + 5;
}

long f029(long x) {
    long acc = x;
    acc += f008(x + 1);
    acc += f016(x + 2);
    acc += f018(x + 3);
    acc += f022(x + 4);
    S29_0 s0 = mk29_0(acc);
    bump29_0(&s0, 3);
    acc += probe29_0(&s0);
    acc += read29_0(&s0);
    acc += classify29_0(1, acc, acc);
    acc += accum29_0(7);
    acc += guard29_0(acc);
    S29_1 s1 = mk29_1(acc);
    bump29_1(&s1, 2);
    acc += probe29_1(&s1);
    acc += read29_1(&s1);
    acc += classify29_1(1, acc, acc);
    acc += accum29_1(9);
    acc += guard29_1(acc);
    S29_2 s2 = mk29_2(acc);
    bump29_2(&s2, 2);
    acc += probe29_2(&s2);
    acc += read29_2(&s2);
    acc += classify29_2(1, acc, acc);
    acc += accum29_2(7);
    acc += guard29_2(acc);
    return clampi(acc);
}
