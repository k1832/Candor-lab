/* GENERATED C mirror of reference module m067. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S67_0;

static S67_0 mk67_0(long a) {
    S67_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe67_0(const S67_0 *s) {
    return s->a + s->n0;
}
static long read67_0(const S67_0 *s) {
    return s->a * 5;
}
static void bump67_0(S67_0 *s, long d) {
    s->a = s->a + d;
}
static long classify67_0(int tag, long a, long b) {
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
static long accum67_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard67_0(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S67_1;

static S67_1 mk67_1(long a) {
    S67_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe67_1(const S67_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read67_1(const S67_1 *s) {
    return s->a * 6;
}
static void bump67_1(S67_1 *s, long d) {
    s->a = s->a + d;
}
static long classify67_1(int tag, long a, long b) {
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
static long accum67_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard67_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S67_2;

static S67_2 mk67_2(long a) {
    S67_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe67_2(const S67_2 *s) {
    return s->a + s->n0;
}
static long read67_2(const S67_2 *s) {
    return s->a * 4;
}
static void bump67_2(S67_2 *s, long d) {
    s->a = s->a + d;
}
static long classify67_2(int tag, long a, long b) {
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
static long accum67_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard67_2(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S67_3;

static S67_3 mk67_3(long a) {
    S67_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe67_3(const S67_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read67_3(const S67_3 *s) {
    return s->a * 4;
}
static void bump67_3(S67_3 *s, long d) {
    s->a = s->a + d;
}
static long classify67_3(int tag, long a, long b) {
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
static long accum67_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard67_3(long x) {
    return x + 2;
}

long f067(long x) {
    long acc = x;
    acc += f007(x + 1);
    S67_0 s0 = mk67_0(acc);
    bump67_0(&s0, 3);
    acc += probe67_0(&s0);
    acc += read67_0(&s0);
    acc += classify67_0(1, acc, acc);
    acc += accum67_0(5);
    acc += guard67_0(acc);
    S67_1 s1 = mk67_1(acc);
    bump67_1(&s1, 3);
    acc += probe67_1(&s1);
    acc += read67_1(&s1);
    acc += classify67_1(1, acc, acc);
    acc += accum67_1(3);
    acc += guard67_1(acc);
    S67_2 s2 = mk67_2(acc);
    bump67_2(&s2, 4);
    acc += probe67_2(&s2);
    acc += read67_2(&s2);
    acc += classify67_2(1, acc, acc);
    acc += accum67_2(3);
    acc += guard67_2(acc);
    S67_3 s3 = mk67_3(acc);
    bump67_3(&s3, 5);
    acc += probe67_3(&s3);
    acc += read67_3(&s3);
    acc += classify67_3(1, acc, acc);
    acc += accum67_3(5);
    acc += guard67_3(acc);
    return clampi(acc);
}
