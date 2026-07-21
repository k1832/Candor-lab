/* GENERATED C mirror of reference module m176. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S176_0;

static S176_0 mk176_0(long a) {
    S176_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe176_0(const S176_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read176_0(const S176_0 *s) {
    return s->a * 4;
}
static void bump176_0(S176_0 *s, long d) {
    s->a = s->a + d;
}
static long classify176_0(int tag, long a, long b) {
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
static long accum176_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard176_0(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S176_1;

static S176_1 mk176_1(long a) {
    S176_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe176_1(const S176_1 *s) {
    return s->a + s->n0;
}
static long read176_1(const S176_1 *s) {
    return s->a * 3;
}
static void bump176_1(S176_1 *s, long d) {
    s->a = s->a + d;
}
static long classify176_1(int tag, long a, long b) {
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
static long accum176_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard176_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S176_2;

static S176_2 mk176_2(long a) {
    S176_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe176_2(const S176_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read176_2(const S176_2 *s) {
    return s->a * 2;
}
static void bump176_2(S176_2 *s, long d) {
    s->a = s->a + d;
}
static long classify176_2(int tag, long a, long b) {
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
static long accum176_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard176_2(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S176_3;

static S176_3 mk176_3(long a) {
    S176_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe176_3(const S176_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read176_3(const S176_3 *s) {
    return s->a * 6;
}
static void bump176_3(S176_3 *s, long d) {
    s->a = s->a + d;
}
static long classify176_3(int tag, long a, long b) {
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
static long accum176_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard176_3(long x) {
    return x + 8;
}

long f176(long x) {
    long acc = x;
    acc += f071(x + 1);
    acc += f087(x + 2);
    acc += f141(x + 3);
    acc += f158(x + 4);
    S176_0 s0 = mk176_0(acc);
    bump176_0(&s0, 4);
    acc += probe176_0(&s0);
    acc += read176_0(&s0);
    acc += classify176_0(1, acc, acc);
    acc += accum176_0(3);
    acc += guard176_0(acc);
    S176_1 s1 = mk176_1(acc);
    bump176_1(&s1, 2);
    acc += probe176_1(&s1);
    acc += read176_1(&s1);
    acc += classify176_1(1, acc, acc);
    acc += accum176_1(7);
    acc += guard176_1(acc);
    S176_2 s2 = mk176_2(acc);
    bump176_2(&s2, 3);
    acc += probe176_2(&s2);
    acc += read176_2(&s2);
    acc += classify176_2(1, acc, acc);
    acc += accum176_2(7);
    acc += guard176_2(acc);
    S176_3 s3 = mk176_3(acc);
    bump176_3(&s3, 7);
    acc += probe176_3(&s3);
    acc += read176_3(&s3);
    acc += classify176_3(1, acc, acc);
    acc += accum176_3(3);
    acc += guard176_3(acc);
    return clampi(acc);
}
