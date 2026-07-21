/* GENERATED C mirror of reference module m025. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S25_0;

static S25_0 mk25_0(long a) {
    S25_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe25_0(const S25_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read25_0(const S25_0 *s) {
    return s->a * 7;
}
static void bump25_0(S25_0 *s, long d) {
    s->a = s->a + d;
}
static long classify25_0(int tag, long a, long b) {
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
static long accum25_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard25_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S25_1;

static S25_1 mk25_1(long a) {
    S25_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe25_1(const S25_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read25_1(const S25_1 *s) {
    return s->a * 4;
}
static void bump25_1(S25_1 *s, long d) {
    s->a = s->a + d;
}
static long classify25_1(int tag, long a, long b) {
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
static long accum25_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard25_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S25_2;

static S25_2 mk25_2(long a) {
    S25_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe25_2(const S25_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read25_2(const S25_2 *s) {
    return s->a * 6;
}
static void bump25_2(S25_2 *s, long d) {
    s->a = s->a + d;
}
static long classify25_2(int tag, long a, long b) {
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
static long accum25_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard25_2(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S25_3;

static S25_3 mk25_3(long a) {
    S25_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe25_3(const S25_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read25_3(const S25_3 *s) {
    return s->a * 5;
}
static void bump25_3(S25_3 *s, long d) {
    s->a = s->a + d;
}
static long classify25_3(int tag, long a, long b) {
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
static long accum25_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard25_3(long x) {
    return x + 9;
}

long f025(long x) {
    long acc = x;
    acc += f006(x + 1);
    acc += f009(x + 2);
    acc += f022(x + 3);
    S25_0 s0 = mk25_0(acc);
    bump25_0(&s0, 7);
    acc += probe25_0(&s0);
    acc += read25_0(&s0);
    acc += classify25_0(1, acc, acc);
    acc += accum25_0(9);
    acc += guard25_0(acc);
    S25_1 s1 = mk25_1(acc);
    bump25_1(&s1, 7);
    acc += probe25_1(&s1);
    acc += read25_1(&s1);
    acc += classify25_1(1, acc, acc);
    acc += accum25_1(6);
    acc += guard25_1(acc);
    S25_2 s2 = mk25_2(acc);
    bump25_2(&s2, 5);
    acc += probe25_2(&s2);
    acc += read25_2(&s2);
    acc += classify25_2(1, acc, acc);
    acc += accum25_2(7);
    acc += guard25_2(acc);
    S25_3 s3 = mk25_3(acc);
    bump25_3(&s3, 8);
    acc += probe25_3(&s3);
    acc += read25_3(&s3);
    acc += classify25_3(1, acc, acc);
    acc += accum25_3(3);
    acc += guard25_3(acc);
    return clampi(acc);
}
