/* GENERATED C mirror of reference module m144. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S144_0;

static S144_0 mk144_0(long a) {
    S144_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe144_0(const S144_0 *s) {
    return s->a + s->n0;
}
static long read144_0(const S144_0 *s) {
    return s->a * 6;
}
static void bump144_0(S144_0 *s, long d) {
    s->a = s->a + d;
}
static long classify144_0(int tag, long a, long b) {
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
static long accum144_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard144_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S144_1;

static S144_1 mk144_1(long a) {
    S144_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe144_1(const S144_1 *s) {
    return s->a + s->n0;
}
static long read144_1(const S144_1 *s) {
    return s->a * 4;
}
static void bump144_1(S144_1 *s, long d) {
    s->a = s->a + d;
}
static long classify144_1(int tag, long a, long b) {
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
static long accum144_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard144_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S144_2;

static S144_2 mk144_2(long a) {
    S144_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe144_2(const S144_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read144_2(const S144_2 *s) {
    return s->a * 6;
}
static void bump144_2(S144_2 *s, long d) {
    s->a = s->a + d;
}
static long classify144_2(int tag, long a, long b) {
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
static long accum144_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard144_2(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S144_3;

static S144_3 mk144_3(long a) {
    S144_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe144_3(const S144_3 *s) {
    return s->a + s->n0;
}
static long read144_3(const S144_3 *s) {
    return s->a * 2;
}
static void bump144_3(S144_3 *s, long d) {
    s->a = s->a + d;
}
static long classify144_3(int tag, long a, long b) {
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
static long accum144_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard144_3(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S144_4;

static S144_4 mk144_4(long a) {
    S144_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe144_4(const S144_4 *s) {
    return s->a + s->n0;
}
static long read144_4(const S144_4 *s) {
    return s->a * 5;
}
static void bump144_4(S144_4 *s, long d) {
    s->a = s->a + d;
}
static long classify144_4(int tag, long a, long b) {
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
static long accum144_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard144_4(long x) {
    return x + 9;
}

long f144(long x) {
    long acc = x;
    acc += f109(x + 1);
    S144_0 s0 = mk144_0(acc);
    bump144_0(&s0, 4);
    acc += probe144_0(&s0);
    acc += read144_0(&s0);
    acc += classify144_0(1, acc, acc);
    acc += accum144_0(9);
    acc += guard144_0(acc);
    S144_1 s1 = mk144_1(acc);
    bump144_1(&s1, 9);
    acc += probe144_1(&s1);
    acc += read144_1(&s1);
    acc += classify144_1(1, acc, acc);
    acc += accum144_1(3);
    acc += guard144_1(acc);
    S144_2 s2 = mk144_2(acc);
    bump144_2(&s2, 8);
    acc += probe144_2(&s2);
    acc += read144_2(&s2);
    acc += classify144_2(1, acc, acc);
    acc += accum144_2(7);
    acc += guard144_2(acc);
    S144_3 s3 = mk144_3(acc);
    bump144_3(&s3, 4);
    acc += probe144_3(&s3);
    acc += read144_3(&s3);
    acc += classify144_3(1, acc, acc);
    acc += accum144_3(7);
    acc += guard144_3(acc);
    S144_4 s4 = mk144_4(acc);
    bump144_4(&s4, 7);
    acc += probe144_4(&s4);
    acc += read144_4(&s4);
    acc += classify144_4(1, acc, acc);
    acc += accum144_4(9);
    acc += guard144_4(acc);
    return clampi(acc);
}
