/* GENERATED C mirror of reference module m065. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S65_0;

static S65_0 mk65_0(long a) {
    S65_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe65_0(const S65_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read65_0(const S65_0 *s) {
    return s->a * 7;
}
static void bump65_0(S65_0 *s, long d) {
    s->a = s->a + d;
}
static long classify65_0(int tag, long a, long b) {
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
static long accum65_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard65_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S65_1;

static S65_1 mk65_1(long a) {
    S65_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe65_1(const S65_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read65_1(const S65_1 *s) {
    return s->a * 3;
}
static void bump65_1(S65_1 *s, long d) {
    s->a = s->a + d;
}
static long classify65_1(int tag, long a, long b) {
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
static long accum65_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard65_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S65_2;

static S65_2 mk65_2(long a) {
    S65_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe65_2(const S65_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read65_2(const S65_2 *s) {
    return s->a * 6;
}
static void bump65_2(S65_2 *s, long d) {
    s->a = s->a + d;
}
static long classify65_2(int tag, long a, long b) {
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
static long accum65_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard65_2(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S65_3;

static S65_3 mk65_3(long a) {
    S65_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe65_3(const S65_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read65_3(const S65_3 *s) {
    return s->a * 6;
}
static void bump65_3(S65_3 *s, long d) {
    s->a = s->a + d;
}
static long classify65_3(int tag, long a, long b) {
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
static long accum65_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard65_3(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S65_4;

static S65_4 mk65_4(long a) {
    S65_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe65_4(const S65_4 *s) {
    return s->a + s->n0;
}
static long read65_4(const S65_4 *s) {
    return s->a * 3;
}
static void bump65_4(S65_4 *s, long d) {
    s->a = s->a + d;
}
static long classify65_4(int tag, long a, long b) {
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
static long accum65_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard65_4(long x) {
    return x + 6;
}

long f065(long x) {
    long acc = x;
    acc += f045(x + 1);
    S65_0 s0 = mk65_0(acc);
    bump65_0(&s0, 2);
    acc += probe65_0(&s0);
    acc += read65_0(&s0);
    acc += classify65_0(1, acc, acc);
    acc += accum65_0(7);
    acc += guard65_0(acc);
    S65_1 s1 = mk65_1(acc);
    bump65_1(&s1, 5);
    acc += probe65_1(&s1);
    acc += read65_1(&s1);
    acc += classify65_1(1, acc, acc);
    acc += accum65_1(8);
    acc += guard65_1(acc);
    S65_2 s2 = mk65_2(acc);
    bump65_2(&s2, 8);
    acc += probe65_2(&s2);
    acc += read65_2(&s2);
    acc += classify65_2(1, acc, acc);
    acc += accum65_2(6);
    acc += guard65_2(acc);
    S65_3 s3 = mk65_3(acc);
    bump65_3(&s3, 1);
    acc += probe65_3(&s3);
    acc += read65_3(&s3);
    acc += classify65_3(1, acc, acc);
    acc += accum65_3(4);
    acc += guard65_3(acc);
    S65_4 s4 = mk65_4(acc);
    bump65_4(&s4, 7);
    acc += probe65_4(&s4);
    acc += read65_4(&s4);
    acc += classify65_4(1, acc, acc);
    acc += accum65_4(7);
    acc += guard65_4(acc);
    return clampi(acc);
}
