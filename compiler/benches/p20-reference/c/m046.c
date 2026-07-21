/* GENERATED C mirror of reference module m046. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S46_0;

static S46_0 mk46_0(long a) {
    S46_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe46_0(const S46_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read46_0(const S46_0 *s) {
    return s->a * 6;
}
static void bump46_0(S46_0 *s, long d) {
    s->a = s->a + d;
}
static long classify46_0(int tag, long a, long b) {
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
static long accum46_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard46_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S46_1;

static S46_1 mk46_1(long a) {
    S46_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe46_1(const S46_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read46_1(const S46_1 *s) {
    return s->a * 2;
}
static void bump46_1(S46_1 *s, long d) {
    s->a = s->a + d;
}
static long classify46_1(int tag, long a, long b) {
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
static long accum46_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard46_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S46_2;

static S46_2 mk46_2(long a) {
    S46_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe46_2(const S46_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read46_2(const S46_2 *s) {
    return s->a * 7;
}
static void bump46_2(S46_2 *s, long d) {
    s->a = s->a + d;
}
static long classify46_2(int tag, long a, long b) {
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
static long accum46_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard46_2(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S46_3;

static S46_3 mk46_3(long a) {
    S46_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe46_3(const S46_3 *s) {
    return s->a + s->n0;
}
static long read46_3(const S46_3 *s) {
    return s->a * 3;
}
static void bump46_3(S46_3 *s, long d) {
    s->a = s->a + d;
}
static long classify46_3(int tag, long a, long b) {
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
static long accum46_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard46_3(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S46_4;

static S46_4 mk46_4(long a) {
    S46_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe46_4(const S46_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read46_4(const S46_4 *s) {
    return s->a * 6;
}
static void bump46_4(S46_4 *s, long d) {
    s->a = s->a + d;
}
static long classify46_4(int tag, long a, long b) {
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
static long accum46_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard46_4(long x) {
    return x + 5;
}

long f046(long x) {
    long acc = x;
    acc += f011(x + 1);
    S46_0 s0 = mk46_0(acc);
    bump46_0(&s0, 8);
    acc += probe46_0(&s0);
    acc += read46_0(&s0);
    acc += classify46_0(1, acc, acc);
    acc += accum46_0(9);
    acc += guard46_0(acc);
    S46_1 s1 = mk46_1(acc);
    bump46_1(&s1, 7);
    acc += probe46_1(&s1);
    acc += read46_1(&s1);
    acc += classify46_1(1, acc, acc);
    acc += accum46_1(7);
    acc += guard46_1(acc);
    S46_2 s2 = mk46_2(acc);
    bump46_2(&s2, 2);
    acc += probe46_2(&s2);
    acc += read46_2(&s2);
    acc += classify46_2(1, acc, acc);
    acc += accum46_2(7);
    acc += guard46_2(acc);
    S46_3 s3 = mk46_3(acc);
    bump46_3(&s3, 2);
    acc += probe46_3(&s3);
    acc += read46_3(&s3);
    acc += classify46_3(1, acc, acc);
    acc += accum46_3(8);
    acc += guard46_3(acc);
    S46_4 s4 = mk46_4(acc);
    bump46_4(&s4, 6);
    acc += probe46_4(&s4);
    acc += read46_4(&s4);
    acc += classify46_4(1, acc, acc);
    acc += accum46_4(9);
    acc += guard46_4(acc);
    return clampi(acc);
}
