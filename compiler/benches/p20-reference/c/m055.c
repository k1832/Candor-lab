/* GENERATED C mirror of reference module m055. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S55_0;

static S55_0 mk55_0(long a) {
    S55_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe55_0(const S55_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read55_0(const S55_0 *s) {
    return s->a * 5;
}
static void bump55_0(S55_0 *s, long d) {
    s->a = s->a + d;
}
static long classify55_0(int tag, long a, long b) {
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
static long accum55_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard55_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S55_1;

static S55_1 mk55_1(long a) {
    S55_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe55_1(const S55_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read55_1(const S55_1 *s) {
    return s->a * 2;
}
static void bump55_1(S55_1 *s, long d) {
    s->a = s->a + d;
}
static long classify55_1(int tag, long a, long b) {
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
static long accum55_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard55_1(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S55_2;

static S55_2 mk55_2(long a) {
    S55_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe55_2(const S55_2 *s) {
    return s->a + s->n0;
}
static long read55_2(const S55_2 *s) {
    return s->a * 3;
}
static void bump55_2(S55_2 *s, long d) {
    s->a = s->a + d;
}
static long classify55_2(int tag, long a, long b) {
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
static long accum55_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard55_2(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S55_3;

static S55_3 mk55_3(long a) {
    S55_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe55_3(const S55_3 *s) {
    return s->a + s->n0;
}
static long read55_3(const S55_3 *s) {
    return s->a * 2;
}
static void bump55_3(S55_3 *s, long d) {
    s->a = s->a + d;
}
static long classify55_3(int tag, long a, long b) {
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
static long accum55_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard55_3(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S55_4;

static S55_4 mk55_4(long a) {
    S55_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe55_4(const S55_4 *s) {
    return s->a + s->n0;
}
static long read55_4(const S55_4 *s) {
    return s->a * 6;
}
static void bump55_4(S55_4 *s, long d) {
    s->a = s->a + d;
}
static long classify55_4(int tag, long a, long b) {
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
static long accum55_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard55_4(long x) {
    return x + 9;
}

long f055(long x) {
    long acc = x;
    acc += f043(x + 1);
    S55_0 s0 = mk55_0(acc);
    bump55_0(&s0, 1);
    acc += probe55_0(&s0);
    acc += read55_0(&s0);
    acc += classify55_0(1, acc, acc);
    acc += accum55_0(5);
    acc += guard55_0(acc);
    S55_1 s1 = mk55_1(acc);
    bump55_1(&s1, 1);
    acc += probe55_1(&s1);
    acc += read55_1(&s1);
    acc += classify55_1(1, acc, acc);
    acc += accum55_1(7);
    acc += guard55_1(acc);
    S55_2 s2 = mk55_2(acc);
    bump55_2(&s2, 8);
    acc += probe55_2(&s2);
    acc += read55_2(&s2);
    acc += classify55_2(1, acc, acc);
    acc += accum55_2(6);
    acc += guard55_2(acc);
    S55_3 s3 = mk55_3(acc);
    bump55_3(&s3, 9);
    acc += probe55_3(&s3);
    acc += read55_3(&s3);
    acc += classify55_3(1, acc, acc);
    acc += accum55_3(8);
    acc += guard55_3(acc);
    S55_4 s4 = mk55_4(acc);
    bump55_4(&s4, 3);
    acc += probe55_4(&s4);
    acc += read55_4(&s4);
    acc += classify55_4(1, acc, acc);
    acc += accum55_4(8);
    acc += guard55_4(acc);
    return clampi(acc);
}
