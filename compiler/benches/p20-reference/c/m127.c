/* GENERATED C mirror of reference module m127. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S127_0;

static S127_0 mk127_0(long a) {
    S127_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe127_0(const S127_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read127_0(const S127_0 *s) {
    return s->a * 7;
}
static void bump127_0(S127_0 *s, long d) {
    s->a = s->a + d;
}
static long classify127_0(int tag, long a, long b) {
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
static long accum127_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard127_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S127_1;

static S127_1 mk127_1(long a) {
    S127_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe127_1(const S127_1 *s) {
    return s->a + s->n0;
}
static long read127_1(const S127_1 *s) {
    return s->a * 4;
}
static void bump127_1(S127_1 *s, long d) {
    s->a = s->a + d;
}
static long classify127_1(int tag, long a, long b) {
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
static long accum127_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard127_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S127_2;

static S127_2 mk127_2(long a) {
    S127_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe127_2(const S127_2 *s) {
    return s->a + s->n0;
}
static long read127_2(const S127_2 *s) {
    return s->a * 4;
}
static void bump127_2(S127_2 *s, long d) {
    s->a = s->a + d;
}
static long classify127_2(int tag, long a, long b) {
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
static long accum127_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard127_2(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S127_3;

static S127_3 mk127_3(long a) {
    S127_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe127_3(const S127_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read127_3(const S127_3 *s) {
    return s->a * 2;
}
static void bump127_3(S127_3 *s, long d) {
    s->a = s->a + d;
}
static long classify127_3(int tag, long a, long b) {
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
static long accum127_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard127_3(long x) {
    return x + 8;
}

long f127(long x) {
    long acc = x;
    acc += f046(x + 1);
    S127_0 s0 = mk127_0(acc);
    bump127_0(&s0, 1);
    acc += probe127_0(&s0);
    acc += read127_0(&s0);
    acc += classify127_0(1, acc, acc);
    acc += accum127_0(5);
    acc += guard127_0(acc);
    S127_1 s1 = mk127_1(acc);
    bump127_1(&s1, 8);
    acc += probe127_1(&s1);
    acc += read127_1(&s1);
    acc += classify127_1(1, acc, acc);
    acc += accum127_1(3);
    acc += guard127_1(acc);
    S127_2 s2 = mk127_2(acc);
    bump127_2(&s2, 1);
    acc += probe127_2(&s2);
    acc += read127_2(&s2);
    acc += classify127_2(1, acc, acc);
    acc += accum127_2(4);
    acc += guard127_2(acc);
    S127_3 s3 = mk127_3(acc);
    bump127_3(&s3, 4);
    acc += probe127_3(&s3);
    acc += read127_3(&s3);
    acc += classify127_3(1, acc, acc);
    acc += accum127_3(8);
    acc += guard127_3(acc);
    return clampi(acc);
}
