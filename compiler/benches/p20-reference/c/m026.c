/* GENERATED C mirror of reference module m026. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S26_0;

static S26_0 mk26_0(long a) {
    S26_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe26_0(const S26_0 *s) {
    return s->a + s->n0;
}
static long read26_0(const S26_0 *s) {
    return s->a * 4;
}
static void bump26_0(S26_0 *s, long d) {
    s->a = s->a + d;
}
static long classify26_0(int tag, long a, long b) {
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
static long accum26_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard26_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S26_1;

static S26_1 mk26_1(long a) {
    S26_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe26_1(const S26_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read26_1(const S26_1 *s) {
    return s->a * 3;
}
static void bump26_1(S26_1 *s, long d) {
    s->a = s->a + d;
}
static long classify26_1(int tag, long a, long b) {
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
static long accum26_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard26_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S26_2;

static S26_2 mk26_2(long a) {
    S26_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe26_2(const S26_2 *s) {
    return s->a + s->n0;
}
static long read26_2(const S26_2 *s) {
    return s->a * 5;
}
static void bump26_2(S26_2 *s, long d) {
    s->a = s->a + d;
}
static long classify26_2(int tag, long a, long b) {
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
static long accum26_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard26_2(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S26_3;

static S26_3 mk26_3(long a) {
    S26_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe26_3(const S26_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read26_3(const S26_3 *s) {
    return s->a * 3;
}
static void bump26_3(S26_3 *s, long d) {
    s->a = s->a + d;
}
static long classify26_3(int tag, long a, long b) {
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
static long accum26_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard26_3(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S26_4;

static S26_4 mk26_4(long a) {
    S26_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe26_4(const S26_4 *s) {
    return s->a + s->n0;
}
static long read26_4(const S26_4 *s) {
    return s->a * 6;
}
static void bump26_4(S26_4 *s, long d) {
    s->a = s->a + d;
}
static long classify26_4(int tag, long a, long b) {
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
static long accum26_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard26_4(long x) {
    return x + 9;
}

long f026(long x) {
    long acc = x;
    acc += f005(x + 1);
    acc += f012(x + 2);
    acc += f014(x + 3);
    S26_0 s0 = mk26_0(acc);
    bump26_0(&s0, 3);
    acc += probe26_0(&s0);
    acc += read26_0(&s0);
    acc += classify26_0(1, acc, acc);
    acc += accum26_0(8);
    acc += guard26_0(acc);
    S26_1 s1 = mk26_1(acc);
    bump26_1(&s1, 7);
    acc += probe26_1(&s1);
    acc += read26_1(&s1);
    acc += classify26_1(1, acc, acc);
    acc += accum26_1(7);
    acc += guard26_1(acc);
    S26_2 s2 = mk26_2(acc);
    bump26_2(&s2, 5);
    acc += probe26_2(&s2);
    acc += read26_2(&s2);
    acc += classify26_2(1, acc, acc);
    acc += accum26_2(4);
    acc += guard26_2(acc);
    S26_3 s3 = mk26_3(acc);
    bump26_3(&s3, 7);
    acc += probe26_3(&s3);
    acc += read26_3(&s3);
    acc += classify26_3(1, acc, acc);
    acc += accum26_3(5);
    acc += guard26_3(acc);
    S26_4 s4 = mk26_4(acc);
    bump26_4(&s4, 9);
    acc += probe26_4(&s4);
    acc += read26_4(&s4);
    acc += classify26_4(1, acc, acc);
    acc += accum26_4(9);
    acc += guard26_4(acc);
    return clampi(acc);
}
