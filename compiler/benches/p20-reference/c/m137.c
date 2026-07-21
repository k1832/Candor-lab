/* GENERATED C mirror of reference module m137. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S137_0;

static S137_0 mk137_0(long a) {
    S137_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe137_0(const S137_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read137_0(const S137_0 *s) {
    return s->a * 6;
}
static void bump137_0(S137_0 *s, long d) {
    s->a = s->a + d;
}
static long classify137_0(int tag, long a, long b) {
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
static long accum137_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard137_0(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S137_1;

static S137_1 mk137_1(long a) {
    S137_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe137_1(const S137_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read137_1(const S137_1 *s) {
    return s->a * 5;
}
static void bump137_1(S137_1 *s, long d) {
    s->a = s->a + d;
}
static long classify137_1(int tag, long a, long b) {
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
static long accum137_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard137_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S137_2;

static S137_2 mk137_2(long a) {
    S137_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe137_2(const S137_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read137_2(const S137_2 *s) {
    return s->a * 3;
}
static void bump137_2(S137_2 *s, long d) {
    s->a = s->a + d;
}
static long classify137_2(int tag, long a, long b) {
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
static long accum137_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard137_2(long x) {
    return x + 3;
}

long f137(long x) {
    long acc = x;
    acc += f070(x + 1);
    acc += f081(x + 2);
    S137_0 s0 = mk137_0(acc);
    bump137_0(&s0, 7);
    acc += probe137_0(&s0);
    acc += read137_0(&s0);
    acc += classify137_0(1, acc, acc);
    acc += accum137_0(6);
    acc += guard137_0(acc);
    S137_1 s1 = mk137_1(acc);
    bump137_1(&s1, 6);
    acc += probe137_1(&s1);
    acc += read137_1(&s1);
    acc += classify137_1(1, acc, acc);
    acc += accum137_1(4);
    acc += guard137_1(acc);
    S137_2 s2 = mk137_2(acc);
    bump137_2(&s2, 2);
    acc += probe137_2(&s2);
    acc += read137_2(&s2);
    acc += classify137_2(1, acc, acc);
    acc += accum137_2(6);
    acc += guard137_2(acc);
    return clampi(acc);
}
