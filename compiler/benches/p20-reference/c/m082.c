/* GENERATED C mirror of reference module m082. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S82_0;

static S82_0 mk82_0(long a) {
    S82_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe82_0(const S82_0 *s) {
    return s->a + s->n0;
}
static long read82_0(const S82_0 *s) {
    return s->a * 4;
}
static void bump82_0(S82_0 *s, long d) {
    s->a = s->a + d;
}
static long classify82_0(int tag, long a, long b) {
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
static long accum82_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard82_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S82_1;

static S82_1 mk82_1(long a) {
    S82_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe82_1(const S82_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read82_1(const S82_1 *s) {
    return s->a * 4;
}
static void bump82_1(S82_1 *s, long d) {
    s->a = s->a + d;
}
static long classify82_1(int tag, long a, long b) {
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
static long accum82_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard82_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S82_2;

static S82_2 mk82_2(long a) {
    S82_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe82_2(const S82_2 *s) {
    return s->a + s->n0;
}
static long read82_2(const S82_2 *s) {
    return s->a * 5;
}
static void bump82_2(S82_2 *s, long d) {
    s->a = s->a + d;
}
static long classify82_2(int tag, long a, long b) {
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
static long accum82_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard82_2(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S82_3;

static S82_3 mk82_3(long a) {
    S82_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe82_3(const S82_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read82_3(const S82_3 *s) {
    return s->a * 5;
}
static void bump82_3(S82_3 *s, long d) {
    s->a = s->a + d;
}
static long classify82_3(int tag, long a, long b) {
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
static long accum82_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard82_3(long x) {
    return x + 8;
}

long f082(long x) {
    long acc = x;
    acc += f056(x + 1);
    S82_0 s0 = mk82_0(acc);
    bump82_0(&s0, 5);
    acc += probe82_0(&s0);
    acc += read82_0(&s0);
    acc += classify82_0(1, acc, acc);
    acc += accum82_0(7);
    acc += guard82_0(acc);
    S82_1 s1 = mk82_1(acc);
    bump82_1(&s1, 1);
    acc += probe82_1(&s1);
    acc += read82_1(&s1);
    acc += classify82_1(1, acc, acc);
    acc += accum82_1(8);
    acc += guard82_1(acc);
    S82_2 s2 = mk82_2(acc);
    bump82_2(&s2, 6);
    acc += probe82_2(&s2);
    acc += read82_2(&s2);
    acc += classify82_2(1, acc, acc);
    acc += accum82_2(8);
    acc += guard82_2(acc);
    S82_3 s3 = mk82_3(acc);
    bump82_3(&s3, 6);
    acc += probe82_3(&s3);
    acc += read82_3(&s3);
    acc += classify82_3(1, acc, acc);
    acc += accum82_3(9);
    acc += guard82_3(acc);
    return clampi(acc);
}
