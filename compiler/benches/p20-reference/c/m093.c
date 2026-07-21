/* GENERATED C mirror of reference module m093. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S93_0;

static S93_0 mk93_0(long a) {
    S93_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe93_0(const S93_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read93_0(const S93_0 *s) {
    return s->a * 5;
}
static void bump93_0(S93_0 *s, long d) {
    s->a = s->a + d;
}
static long classify93_0(int tag, long a, long b) {
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
static long accum93_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard93_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S93_1;

static S93_1 mk93_1(long a) {
    S93_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe93_1(const S93_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read93_1(const S93_1 *s) {
    return s->a * 5;
}
static void bump93_1(S93_1 *s, long d) {
    s->a = s->a + d;
}
static long classify93_1(int tag, long a, long b) {
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
static long accum93_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard93_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S93_2;

static S93_2 mk93_2(long a) {
    S93_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe93_2(const S93_2 *s) {
    return s->a + s->n0;
}
static long read93_2(const S93_2 *s) {
    return s->a * 5;
}
static void bump93_2(S93_2 *s, long d) {
    s->a = s->a + d;
}
static long classify93_2(int tag, long a, long b) {
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
static long accum93_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard93_2(long x) {
    return x + 5;
}

long f093(long x) {
    long acc = x;
    acc += f045(x + 1);
    acc += f068(x + 2);
    S93_0 s0 = mk93_0(acc);
    bump93_0(&s0, 9);
    acc += probe93_0(&s0);
    acc += read93_0(&s0);
    acc += classify93_0(1, acc, acc);
    acc += accum93_0(3);
    acc += guard93_0(acc);
    S93_1 s1 = mk93_1(acc);
    bump93_1(&s1, 5);
    acc += probe93_1(&s1);
    acc += read93_1(&s1);
    acc += classify93_1(1, acc, acc);
    acc += accum93_1(6);
    acc += guard93_1(acc);
    S93_2 s2 = mk93_2(acc);
    bump93_2(&s2, 1);
    acc += probe93_2(&s2);
    acc += read93_2(&s2);
    acc += classify93_2(1, acc, acc);
    acc += accum93_2(9);
    acc += guard93_2(acc);
    return clampi(acc);
}
