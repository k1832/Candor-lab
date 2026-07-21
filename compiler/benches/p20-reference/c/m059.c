/* GENERATED C mirror of reference module m059. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S59_0;

static S59_0 mk59_0(long a) {
    S59_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe59_0(const S59_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read59_0(const S59_0 *s) {
    return s->a * 3;
}
static void bump59_0(S59_0 *s, long d) {
    s->a = s->a + d;
}
static long classify59_0(int tag, long a, long b) {
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
static long accum59_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard59_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S59_1;

static S59_1 mk59_1(long a) {
    S59_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe59_1(const S59_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read59_1(const S59_1 *s) {
    return s->a * 7;
}
static void bump59_1(S59_1 *s, long d) {
    s->a = s->a + d;
}
static long classify59_1(int tag, long a, long b) {
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
static long accum59_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard59_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S59_2;

static S59_2 mk59_2(long a) {
    S59_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe59_2(const S59_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read59_2(const S59_2 *s) {
    return s->a * 4;
}
static void bump59_2(S59_2 *s, long d) {
    s->a = s->a + d;
}
static long classify59_2(int tag, long a, long b) {
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
static long accum59_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard59_2(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S59_3;

static S59_3 mk59_3(long a) {
    S59_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe59_3(const S59_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read59_3(const S59_3 *s) {
    return s->a * 4;
}
static void bump59_3(S59_3 *s, long d) {
    s->a = s->a + d;
}
static long classify59_3(int tag, long a, long b) {
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
static long accum59_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard59_3(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S59_4;

static S59_4 mk59_4(long a) {
    S59_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe59_4(const S59_4 *s) {
    return s->a + s->n0;
}
static long read59_4(const S59_4 *s) {
    return s->a * 6;
}
static void bump59_4(S59_4 *s, long d) {
    s->a = s->a + d;
}
static long classify59_4(int tag, long a, long b) {
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
static long accum59_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard59_4(long x) {
    return x + 9;
}

long f059(long x) {
    long acc = x;
    acc += f002(x + 1);
    acc += f013(x + 2);
    acc += f017(x + 3);
    acc += f025(x + 4);
    S59_0 s0 = mk59_0(acc);
    bump59_0(&s0, 1);
    acc += probe59_0(&s0);
    acc += read59_0(&s0);
    acc += classify59_0(1, acc, acc);
    acc += accum59_0(9);
    acc += guard59_0(acc);
    S59_1 s1 = mk59_1(acc);
    bump59_1(&s1, 6);
    acc += probe59_1(&s1);
    acc += read59_1(&s1);
    acc += classify59_1(1, acc, acc);
    acc += accum59_1(7);
    acc += guard59_1(acc);
    S59_2 s2 = mk59_2(acc);
    bump59_2(&s2, 5);
    acc += probe59_2(&s2);
    acc += read59_2(&s2);
    acc += classify59_2(1, acc, acc);
    acc += accum59_2(9);
    acc += guard59_2(acc);
    S59_3 s3 = mk59_3(acc);
    bump59_3(&s3, 8);
    acc += probe59_3(&s3);
    acc += read59_3(&s3);
    acc += classify59_3(1, acc, acc);
    acc += accum59_3(4);
    acc += guard59_3(acc);
    S59_4 s4 = mk59_4(acc);
    bump59_4(&s4, 4);
    acc += probe59_4(&s4);
    acc += read59_4(&s4);
    acc += classify59_4(1, acc, acc);
    acc += accum59_4(3);
    acc += guard59_4(acc);
    return clampi(acc);
}
