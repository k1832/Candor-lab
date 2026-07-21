/* GENERATED C mirror of reference module m027. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S27_0;

static S27_0 mk27_0(long a) {
    S27_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe27_0(const S27_0 *s) {
    return s->a + s->n0;
}
static long read27_0(const S27_0 *s) {
    return s->a * 7;
}
static void bump27_0(S27_0 *s, long d) {
    s->a = s->a + d;
}
static long classify27_0(int tag, long a, long b) {
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
static long accum27_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard27_0(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S27_1;

static S27_1 mk27_1(long a) {
    S27_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe27_1(const S27_1 *s) {
    return s->a + s->n0;
}
static long read27_1(const S27_1 *s) {
    return s->a * 4;
}
static void bump27_1(S27_1 *s, long d) {
    s->a = s->a + d;
}
static long classify27_1(int tag, long a, long b) {
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
static long accum27_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard27_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S27_2;

static S27_2 mk27_2(long a) {
    S27_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe27_2(const S27_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read27_2(const S27_2 *s) {
    return s->a * 5;
}
static void bump27_2(S27_2 *s, long d) {
    s->a = s->a + d;
}
static long classify27_2(int tag, long a, long b) {
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
static long accum27_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard27_2(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S27_3;

static S27_3 mk27_3(long a) {
    S27_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe27_3(const S27_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read27_3(const S27_3 *s) {
    return s->a * 5;
}
static void bump27_3(S27_3 *s, long d) {
    s->a = s->a + d;
}
static long classify27_3(int tag, long a, long b) {
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
static long accum27_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard27_3(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S27_4;

static S27_4 mk27_4(long a) {
    S27_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe27_4(const S27_4 *s) {
    return s->a + s->n0;
}
static long read27_4(const S27_4 *s) {
    return s->a * 2;
}
static void bump27_4(S27_4 *s, long d) {
    s->a = s->a + d;
}
static long classify27_4(int tag, long a, long b) {
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
static long accum27_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard27_4(long x) {
    return x + 5;
}

long f027(long x) {
    long acc = x;
    acc += f001(x + 1);
    S27_0 s0 = mk27_0(acc);
    bump27_0(&s0, 2);
    acc += probe27_0(&s0);
    acc += read27_0(&s0);
    acc += classify27_0(1, acc, acc);
    acc += accum27_0(8);
    acc += guard27_0(acc);
    S27_1 s1 = mk27_1(acc);
    bump27_1(&s1, 5);
    acc += probe27_1(&s1);
    acc += read27_1(&s1);
    acc += classify27_1(1, acc, acc);
    acc += accum27_1(5);
    acc += guard27_1(acc);
    S27_2 s2 = mk27_2(acc);
    bump27_2(&s2, 3);
    acc += probe27_2(&s2);
    acc += read27_2(&s2);
    acc += classify27_2(1, acc, acc);
    acc += accum27_2(3);
    acc += guard27_2(acc);
    S27_3 s3 = mk27_3(acc);
    bump27_3(&s3, 2);
    acc += probe27_3(&s3);
    acc += read27_3(&s3);
    acc += classify27_3(1, acc, acc);
    acc += accum27_3(5);
    acc += guard27_3(acc);
    S27_4 s4 = mk27_4(acc);
    bump27_4(&s4, 7);
    acc += probe27_4(&s4);
    acc += read27_4(&s4);
    acc += classify27_4(1, acc, acc);
    acc += accum27_4(9);
    acc += guard27_4(acc);
    return clampi(acc);
}
