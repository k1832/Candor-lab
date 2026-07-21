/* GENERATED C mirror of reference module m068. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S68_0;

static S68_0 mk68_0(long a) {
    S68_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe68_0(const S68_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read68_0(const S68_0 *s) {
    return s->a * 2;
}
static void bump68_0(S68_0 *s, long d) {
    s->a = s->a + d;
}
static long classify68_0(int tag, long a, long b) {
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
static long accum68_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard68_0(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S68_1;

static S68_1 mk68_1(long a) {
    S68_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe68_1(const S68_1 *s) {
    return s->a + s->n0;
}
static long read68_1(const S68_1 *s) {
    return s->a * 2;
}
static void bump68_1(S68_1 *s, long d) {
    s->a = s->a + d;
}
static long classify68_1(int tag, long a, long b) {
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
static long accum68_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard68_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S68_2;

static S68_2 mk68_2(long a) {
    S68_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe68_2(const S68_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read68_2(const S68_2 *s) {
    return s->a * 3;
}
static void bump68_2(S68_2 *s, long d) {
    s->a = s->a + d;
}
static long classify68_2(int tag, long a, long b) {
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
static long accum68_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard68_2(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S68_3;

static S68_3 mk68_3(long a) {
    S68_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe68_3(const S68_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read68_3(const S68_3 *s) {
    return s->a * 6;
}
static void bump68_3(S68_3 *s, long d) {
    s->a = s->a + d;
}
static long classify68_3(int tag, long a, long b) {
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
static long accum68_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard68_3(long x) {
    return x + 2;
}

long f068(long x) {
    long acc = x;
    acc += f002(x + 1);
    acc += f007(x + 2);
    acc += f041(x + 3);
    S68_0 s0 = mk68_0(acc);
    bump68_0(&s0, 6);
    acc += probe68_0(&s0);
    acc += read68_0(&s0);
    acc += classify68_0(1, acc, acc);
    acc += accum68_0(3);
    acc += guard68_0(acc);
    S68_1 s1 = mk68_1(acc);
    bump68_1(&s1, 1);
    acc += probe68_1(&s1);
    acc += read68_1(&s1);
    acc += classify68_1(1, acc, acc);
    acc += accum68_1(6);
    acc += guard68_1(acc);
    S68_2 s2 = mk68_2(acc);
    bump68_2(&s2, 6);
    acc += probe68_2(&s2);
    acc += read68_2(&s2);
    acc += classify68_2(1, acc, acc);
    acc += accum68_2(5);
    acc += guard68_2(acc);
    S68_3 s3 = mk68_3(acc);
    bump68_3(&s3, 1);
    acc += probe68_3(&s3);
    acc += read68_3(&s3);
    acc += classify68_3(1, acc, acc);
    acc += accum68_3(9);
    acc += guard68_3(acc);
    return clampi(acc);
}
