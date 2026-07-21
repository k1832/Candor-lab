/* GENERATED C mirror of reference module m181. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S181_0;

static S181_0 mk181_0(long a) {
    S181_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe181_0(const S181_0 *s) {
    return s->a + s->n0;
}
static long read181_0(const S181_0 *s) {
    return s->a * 2;
}
static void bump181_0(S181_0 *s, long d) {
    s->a = s->a + d;
}
static long classify181_0(int tag, long a, long b) {
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
static long accum181_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard181_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S181_1;

static S181_1 mk181_1(long a) {
    S181_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe181_1(const S181_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read181_1(const S181_1 *s) {
    return s->a * 4;
}
static void bump181_1(S181_1 *s, long d) {
    s->a = s->a + d;
}
static long classify181_1(int tag, long a, long b) {
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
static long accum181_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard181_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S181_2;

static S181_2 mk181_2(long a) {
    S181_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe181_2(const S181_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read181_2(const S181_2 *s) {
    return s->a * 4;
}
static void bump181_2(S181_2 *s, long d) {
    s->a = s->a + d;
}
static long classify181_2(int tag, long a, long b) {
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
static long accum181_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard181_2(long x) {
    return x + 2;
}

long f181(long x) {
    long acc = x;
    acc += f035(x + 1);
    acc += f121(x + 2);
    acc += f161(x + 3);
    S181_0 s0 = mk181_0(acc);
    bump181_0(&s0, 8);
    acc += probe181_0(&s0);
    acc += read181_0(&s0);
    acc += classify181_0(1, acc, acc);
    acc += accum181_0(4);
    acc += guard181_0(acc);
    S181_1 s1 = mk181_1(acc);
    bump181_1(&s1, 8);
    acc += probe181_1(&s1);
    acc += read181_1(&s1);
    acc += classify181_1(1, acc, acc);
    acc += accum181_1(7);
    acc += guard181_1(acc);
    S181_2 s2 = mk181_2(acc);
    bump181_2(&s2, 7);
    acc += probe181_2(&s2);
    acc += read181_2(&s2);
    acc += classify181_2(1, acc, acc);
    acc += accum181_2(3);
    acc += guard181_2(acc);
    return clampi(acc);
}
