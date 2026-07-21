/* GENERATED C mirror of reference module m112. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S112_0;

static S112_0 mk112_0(long a) {
    S112_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe112_0(const S112_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read112_0(const S112_0 *s) {
    return s->a * 4;
}
static void bump112_0(S112_0 *s, long d) {
    s->a = s->a + d;
}
static long classify112_0(int tag, long a, long b) {
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
static long accum112_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard112_0(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S112_1;

static S112_1 mk112_1(long a) {
    S112_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe112_1(const S112_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read112_1(const S112_1 *s) {
    return s->a * 3;
}
static void bump112_1(S112_1 *s, long d) {
    s->a = s->a + d;
}
static long classify112_1(int tag, long a, long b) {
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
static long accum112_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard112_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S112_2;

static S112_2 mk112_2(long a) {
    S112_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe112_2(const S112_2 *s) {
    return s->a + s->n0;
}
static long read112_2(const S112_2 *s) {
    return s->a * 2;
}
static void bump112_2(S112_2 *s, long d) {
    s->a = s->a + d;
}
static long classify112_2(int tag, long a, long b) {
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
static long accum112_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard112_2(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S112_3;

static S112_3 mk112_3(long a) {
    S112_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe112_3(const S112_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read112_3(const S112_3 *s) {
    return s->a * 7;
}
static void bump112_3(S112_3 *s, long d) {
    s->a = s->a + d;
}
static long classify112_3(int tag, long a, long b) {
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
static long accum112_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard112_3(long x) {
    return x + 4;
}

long f112(long x) {
    long acc = x;
    acc += f001(x + 1);
    acc += f013(x + 2);
    acc += f038(x + 3);
    acc += f049(x + 4);
    S112_0 s0 = mk112_0(acc);
    bump112_0(&s0, 1);
    acc += probe112_0(&s0);
    acc += read112_0(&s0);
    acc += classify112_0(1, acc, acc);
    acc += accum112_0(6);
    acc += guard112_0(acc);
    S112_1 s1 = mk112_1(acc);
    bump112_1(&s1, 9);
    acc += probe112_1(&s1);
    acc += read112_1(&s1);
    acc += classify112_1(1, acc, acc);
    acc += accum112_1(9);
    acc += guard112_1(acc);
    S112_2 s2 = mk112_2(acc);
    bump112_2(&s2, 7);
    acc += probe112_2(&s2);
    acc += read112_2(&s2);
    acc += classify112_2(1, acc, acc);
    acc += accum112_2(9);
    acc += guard112_2(acc);
    S112_3 s3 = mk112_3(acc);
    bump112_3(&s3, 4);
    acc += probe112_3(&s3);
    acc += read112_3(&s3);
    acc += classify112_3(1, acc, acc);
    acc += accum112_3(7);
    acc += guard112_3(acc);
    return clampi(acc);
}
