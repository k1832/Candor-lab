/* GENERATED C mirror of reference module m057. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S57_0;

static S57_0 mk57_0(long a) {
    S57_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe57_0(const S57_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read57_0(const S57_0 *s) {
    return s->a * 5;
}
static void bump57_0(S57_0 *s, long d) {
    s->a = s->a + d;
}
static long classify57_0(int tag, long a, long b) {
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
static long accum57_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard57_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S57_1;

static S57_1 mk57_1(long a) {
    S57_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe57_1(const S57_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read57_1(const S57_1 *s) {
    return s->a * 7;
}
static void bump57_1(S57_1 *s, long d) {
    s->a = s->a + d;
}
static long classify57_1(int tag, long a, long b) {
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
static long accum57_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard57_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S57_2;

static S57_2 mk57_2(long a) {
    S57_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe57_2(const S57_2 *s) {
    return s->a + s->n0;
}
static long read57_2(const S57_2 *s) {
    return s->a * 6;
}
static void bump57_2(S57_2 *s, long d) {
    s->a = s->a + d;
}
static long classify57_2(int tag, long a, long b) {
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
static long accum57_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard57_2(long x) {
    return x + 3;
}

long f057(long x) {
    long acc = x;
    acc += f032(x + 1);
    acc += f046(x + 2);
    S57_0 s0 = mk57_0(acc);
    bump57_0(&s0, 4);
    acc += probe57_0(&s0);
    acc += read57_0(&s0);
    acc += classify57_0(1, acc, acc);
    acc += accum57_0(6);
    acc += guard57_0(acc);
    S57_1 s1 = mk57_1(acc);
    bump57_1(&s1, 6);
    acc += probe57_1(&s1);
    acc += read57_1(&s1);
    acc += classify57_1(1, acc, acc);
    acc += accum57_1(7);
    acc += guard57_1(acc);
    S57_2 s2 = mk57_2(acc);
    bump57_2(&s2, 2);
    acc += probe57_2(&s2);
    acc += read57_2(&s2);
    acc += classify57_2(1, acc, acc);
    acc += accum57_2(3);
    acc += guard57_2(acc);
    return clampi(acc);
}
