/* GENERATED C mirror of reference module m062. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S62_0;

static S62_0 mk62_0(long a) {
    S62_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe62_0(const S62_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read62_0(const S62_0 *s) {
    return s->a * 5;
}
static void bump62_0(S62_0 *s, long d) {
    s->a = s->a + d;
}
static long classify62_0(int tag, long a, long b) {
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
static long accum62_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard62_0(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S62_1;

static S62_1 mk62_1(long a) {
    S62_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe62_1(const S62_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read62_1(const S62_1 *s) {
    return s->a * 6;
}
static void bump62_1(S62_1 *s, long d) {
    s->a = s->a + d;
}
static long classify62_1(int tag, long a, long b) {
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
static long accum62_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard62_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S62_2;

static S62_2 mk62_2(long a) {
    S62_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe62_2(const S62_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read62_2(const S62_2 *s) {
    return s->a * 7;
}
static void bump62_2(S62_2 *s, long d) {
    s->a = s->a + d;
}
static long classify62_2(int tag, long a, long b) {
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
static long accum62_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard62_2(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S62_3;

static S62_3 mk62_3(long a) {
    S62_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe62_3(const S62_3 *s) {
    return s->a + s->n0;
}
static long read62_3(const S62_3 *s) {
    return s->a * 3;
}
static void bump62_3(S62_3 *s, long d) {
    s->a = s->a + d;
}
static long classify62_3(int tag, long a, long b) {
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
static long accum62_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard62_3(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S62_4;

static S62_4 mk62_4(long a) {
    S62_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe62_4(const S62_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read62_4(const S62_4 *s) {
    return s->a * 3;
}
static void bump62_4(S62_4 *s, long d) {
    s->a = s->a + d;
}
static long classify62_4(int tag, long a, long b) {
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
static long accum62_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard62_4(long x) {
    return x + 3;
}

long f062(long x) {
    long acc = x;
    acc += f026(x + 1);
    S62_0 s0 = mk62_0(acc);
    bump62_0(&s0, 2);
    acc += probe62_0(&s0);
    acc += read62_0(&s0);
    acc += classify62_0(1, acc, acc);
    acc += accum62_0(5);
    acc += guard62_0(acc);
    S62_1 s1 = mk62_1(acc);
    bump62_1(&s1, 6);
    acc += probe62_1(&s1);
    acc += read62_1(&s1);
    acc += classify62_1(1, acc, acc);
    acc += accum62_1(9);
    acc += guard62_1(acc);
    S62_2 s2 = mk62_2(acc);
    bump62_2(&s2, 9);
    acc += probe62_2(&s2);
    acc += read62_2(&s2);
    acc += classify62_2(1, acc, acc);
    acc += accum62_2(8);
    acc += guard62_2(acc);
    S62_3 s3 = mk62_3(acc);
    bump62_3(&s3, 6);
    acc += probe62_3(&s3);
    acc += read62_3(&s3);
    acc += classify62_3(1, acc, acc);
    acc += accum62_3(3);
    acc += guard62_3(acc);
    S62_4 s4 = mk62_4(acc);
    bump62_4(&s4, 5);
    acc += probe62_4(&s4);
    acc += read62_4(&s4);
    acc += classify62_4(1, acc, acc);
    acc += accum62_4(9);
    acc += guard62_4(acc);
    return clampi(acc);
}
