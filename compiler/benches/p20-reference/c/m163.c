/* GENERATED C mirror of reference module m163. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S163_0;

static S163_0 mk163_0(long a) {
    S163_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe163_0(const S163_0 *s) {
    return s->a + s->n0;
}
static long read163_0(const S163_0 *s) {
    return s->a * 2;
}
static void bump163_0(S163_0 *s, long d) {
    s->a = s->a + d;
}
static long classify163_0(int tag, long a, long b) {
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
static long accum163_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard163_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S163_1;

static S163_1 mk163_1(long a) {
    S163_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe163_1(const S163_1 *s) {
    return s->a + s->n0;
}
static long read163_1(const S163_1 *s) {
    return s->a * 5;
}
static void bump163_1(S163_1 *s, long d) {
    s->a = s->a + d;
}
static long classify163_1(int tag, long a, long b) {
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
static long accum163_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard163_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S163_2;

static S163_2 mk163_2(long a) {
    S163_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe163_2(const S163_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read163_2(const S163_2 *s) {
    return s->a * 6;
}
static void bump163_2(S163_2 *s, long d) {
    s->a = s->a + d;
}
static long classify163_2(int tag, long a, long b) {
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
static long accum163_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard163_2(long x) {
    return x + 1;
}

long f163(long x) {
    long acc = x;
    acc += f024(x + 1);
    acc += f044(x + 2);
    acc += f103(x + 3);
    S163_0 s0 = mk163_0(acc);
    bump163_0(&s0, 8);
    acc += probe163_0(&s0);
    acc += read163_0(&s0);
    acc += classify163_0(1, acc, acc);
    acc += accum163_0(9);
    acc += guard163_0(acc);
    S163_1 s1 = mk163_1(acc);
    bump163_1(&s1, 6);
    acc += probe163_1(&s1);
    acc += read163_1(&s1);
    acc += classify163_1(1, acc, acc);
    acc += accum163_1(9);
    acc += guard163_1(acc);
    S163_2 s2 = mk163_2(acc);
    bump163_2(&s2, 9);
    acc += probe163_2(&s2);
    acc += read163_2(&s2);
    acc += classify163_2(1, acc, acc);
    acc += accum163_2(4);
    acc += guard163_2(acc);
    return clampi(acc);
}
