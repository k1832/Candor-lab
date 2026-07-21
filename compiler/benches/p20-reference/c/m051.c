/* GENERATED C mirror of reference module m051. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S51_0;

static S51_0 mk51_0(long a) {
    S51_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe51_0(const S51_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read51_0(const S51_0 *s) {
    return s->a * 6;
}
static void bump51_0(S51_0 *s, long d) {
    s->a = s->a + d;
}
static long classify51_0(int tag, long a, long b) {
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
static long accum51_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard51_0(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S51_1;

static S51_1 mk51_1(long a) {
    S51_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe51_1(const S51_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read51_1(const S51_1 *s) {
    return s->a * 4;
}
static void bump51_1(S51_1 *s, long d) {
    s->a = s->a + d;
}
static long classify51_1(int tag, long a, long b) {
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
static long accum51_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard51_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S51_2;

static S51_2 mk51_2(long a) {
    S51_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe51_2(const S51_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read51_2(const S51_2 *s) {
    return s->a * 2;
}
static void bump51_2(S51_2 *s, long d) {
    s->a = s->a + d;
}
static long classify51_2(int tag, long a, long b) {
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
static long accum51_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard51_2(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S51_3;

static S51_3 mk51_3(long a) {
    S51_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe51_3(const S51_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read51_3(const S51_3 *s) {
    return s->a * 6;
}
static void bump51_3(S51_3 *s, long d) {
    s->a = s->a + d;
}
static long classify51_3(int tag, long a, long b) {
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
static long accum51_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard51_3(long x) {
    return x + 7;
}

long f051(long x) {
    long acc = x;
    acc += f022(x + 1);
    acc += f035(x + 2);
    acc += f040(x + 3);
    acc += f047(x + 4);
    S51_0 s0 = mk51_0(acc);
    bump51_0(&s0, 4);
    acc += probe51_0(&s0);
    acc += read51_0(&s0);
    acc += classify51_0(1, acc, acc);
    acc += accum51_0(3);
    acc += guard51_0(acc);
    S51_1 s1 = mk51_1(acc);
    bump51_1(&s1, 9);
    acc += probe51_1(&s1);
    acc += read51_1(&s1);
    acc += classify51_1(1, acc, acc);
    acc += accum51_1(4);
    acc += guard51_1(acc);
    S51_2 s2 = mk51_2(acc);
    bump51_2(&s2, 5);
    acc += probe51_2(&s2);
    acc += read51_2(&s2);
    acc += classify51_2(1, acc, acc);
    acc += accum51_2(9);
    acc += guard51_2(acc);
    S51_3 s3 = mk51_3(acc);
    bump51_3(&s3, 7);
    acc += probe51_3(&s3);
    acc += read51_3(&s3);
    acc += classify51_3(1, acc, acc);
    acc += accum51_3(7);
    acc += guard51_3(acc);
    return clampi(acc);
}
