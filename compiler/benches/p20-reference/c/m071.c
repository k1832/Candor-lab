/* GENERATED C mirror of reference module m071. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S71_0;

static S71_0 mk71_0(long a) {
    S71_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe71_0(const S71_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read71_0(const S71_0 *s) {
    return s->a * 5;
}
static void bump71_0(S71_0 *s, long d) {
    s->a = s->a + d;
}
static long classify71_0(int tag, long a, long b) {
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
static long accum71_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard71_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S71_1;

static S71_1 mk71_1(long a) {
    S71_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe71_1(const S71_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read71_1(const S71_1 *s) {
    return s->a * 5;
}
static void bump71_1(S71_1 *s, long d) {
    s->a = s->a + d;
}
static long classify71_1(int tag, long a, long b) {
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
static long accum71_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard71_1(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S71_2;

static S71_2 mk71_2(long a) {
    S71_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe71_2(const S71_2 *s) {
    return s->a + s->n0;
}
static long read71_2(const S71_2 *s) {
    return s->a * 4;
}
static void bump71_2(S71_2 *s, long d) {
    s->a = s->a + d;
}
static long classify71_2(int tag, long a, long b) {
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
static long accum71_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard71_2(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S71_3;

static S71_3 mk71_3(long a) {
    S71_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe71_3(const S71_3 *s) {
    return s->a + s->n0;
}
static long read71_3(const S71_3 *s) {
    return s->a * 6;
}
static void bump71_3(S71_3 *s, long d) {
    s->a = s->a + d;
}
static long classify71_3(int tag, long a, long b) {
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
static long accum71_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard71_3(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S71_4;

static S71_4 mk71_4(long a) {
    S71_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe71_4(const S71_4 *s) {
    return s->a + s->n0;
}
static long read71_4(const S71_4 *s) {
    return s->a * 3;
}
static void bump71_4(S71_4 *s, long d) {
    s->a = s->a + d;
}
static long classify71_4(int tag, long a, long b) {
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
static long accum71_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard71_4(long x) {
    return x + 2;
}

long f071(long x) {
    long acc = x;
    acc += f001(x + 1);
    S71_0 s0 = mk71_0(acc);
    bump71_0(&s0, 9);
    acc += probe71_0(&s0);
    acc += read71_0(&s0);
    acc += classify71_0(1, acc, acc);
    acc += accum71_0(9);
    acc += guard71_0(acc);
    S71_1 s1 = mk71_1(acc);
    bump71_1(&s1, 5);
    acc += probe71_1(&s1);
    acc += read71_1(&s1);
    acc += classify71_1(1, acc, acc);
    acc += accum71_1(8);
    acc += guard71_1(acc);
    S71_2 s2 = mk71_2(acc);
    bump71_2(&s2, 4);
    acc += probe71_2(&s2);
    acc += read71_2(&s2);
    acc += classify71_2(1, acc, acc);
    acc += accum71_2(3);
    acc += guard71_2(acc);
    S71_3 s3 = mk71_3(acc);
    bump71_3(&s3, 2);
    acc += probe71_3(&s3);
    acc += read71_3(&s3);
    acc += classify71_3(1, acc, acc);
    acc += accum71_3(8);
    acc += guard71_3(acc);
    S71_4 s4 = mk71_4(acc);
    bump71_4(&s4, 7);
    acc += probe71_4(&s4);
    acc += read71_4(&s4);
    acc += classify71_4(1, acc, acc);
    acc += accum71_4(8);
    acc += guard71_4(acc);
    return clampi(acc);
}
