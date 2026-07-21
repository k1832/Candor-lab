/* GENERATED C mirror of reference module m028. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S28_0;

static S28_0 mk28_0(long a) {
    S28_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe28_0(const S28_0 *s) {
    return s->a + s->n0;
}
static long read28_0(const S28_0 *s) {
    return s->a * 2;
}
static void bump28_0(S28_0 *s, long d) {
    s->a = s->a + d;
}
static long classify28_0(int tag, long a, long b) {
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
static long accum28_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard28_0(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S28_1;

static S28_1 mk28_1(long a) {
    S28_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe28_1(const S28_1 *s) {
    return s->a + s->n0;
}
static long read28_1(const S28_1 *s) {
    return s->a * 6;
}
static void bump28_1(S28_1 *s, long d) {
    s->a = s->a + d;
}
static long classify28_1(int tag, long a, long b) {
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
static long accum28_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard28_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S28_2;

static S28_2 mk28_2(long a) {
    S28_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe28_2(const S28_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read28_2(const S28_2 *s) {
    return s->a * 5;
}
static void bump28_2(S28_2 *s, long d) {
    s->a = s->a + d;
}
static long classify28_2(int tag, long a, long b) {
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
static long accum28_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard28_2(long x) {
    return x + 6;
}

long f028(long x) {
    long acc = x;
    acc += f008(x + 1);
    acc += f023(x + 2);
    S28_0 s0 = mk28_0(acc);
    bump28_0(&s0, 2);
    acc += probe28_0(&s0);
    acc += read28_0(&s0);
    acc += classify28_0(1, acc, acc);
    acc += accum28_0(3);
    acc += guard28_0(acc);
    S28_1 s1 = mk28_1(acc);
    bump28_1(&s1, 2);
    acc += probe28_1(&s1);
    acc += read28_1(&s1);
    acc += classify28_1(1, acc, acc);
    acc += accum28_1(6);
    acc += guard28_1(acc);
    S28_2 s2 = mk28_2(acc);
    bump28_2(&s2, 2);
    acc += probe28_2(&s2);
    acc += read28_2(&s2);
    acc += classify28_2(1, acc, acc);
    acc += accum28_2(9);
    acc += guard28_2(acc);
    return clampi(acc);
}
