/* GENERATED C mirror of reference module m010. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S10_0;

static S10_0 mk10_0(long a) {
    S10_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe10_0(const S10_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read10_0(const S10_0 *s) {
    return s->a * 6;
}
static void bump10_0(S10_0 *s, long d) {
    s->a = s->a + d;
}
static long classify10_0(int tag, long a, long b) {
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
static long accum10_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard10_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S10_1;

static S10_1 mk10_1(long a) {
    S10_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe10_1(const S10_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read10_1(const S10_1 *s) {
    return s->a * 6;
}
static void bump10_1(S10_1 *s, long d) {
    s->a = s->a + d;
}
static long classify10_1(int tag, long a, long b) {
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
static long accum10_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard10_1(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S10_2;

static S10_2 mk10_2(long a) {
    S10_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe10_2(const S10_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read10_2(const S10_2 *s) {
    return s->a * 6;
}
static void bump10_2(S10_2 *s, long d) {
    s->a = s->a + d;
}
static long classify10_2(int tag, long a, long b) {
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
static long accum10_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard10_2(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S10_3;

static S10_3 mk10_3(long a) {
    S10_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe10_3(const S10_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read10_3(const S10_3 *s) {
    return s->a * 5;
}
static void bump10_3(S10_3 *s, long d) {
    s->a = s->a + d;
}
static long classify10_3(int tag, long a, long b) {
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
static long accum10_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard10_3(long x) {
    return x + 4;
}

long f010(long x) {
    long acc = x;
    acc += f000(x + 1);
    acc += f003(x + 2);
    S10_0 s0 = mk10_0(acc);
    bump10_0(&s0, 2);
    acc += probe10_0(&s0);
    acc += read10_0(&s0);
    acc += classify10_0(1, acc, acc);
    acc += accum10_0(7);
    acc += guard10_0(acc);
    S10_1 s1 = mk10_1(acc);
    bump10_1(&s1, 4);
    acc += probe10_1(&s1);
    acc += read10_1(&s1);
    acc += classify10_1(1, acc, acc);
    acc += accum10_1(8);
    acc += guard10_1(acc);
    S10_2 s2 = mk10_2(acc);
    bump10_2(&s2, 8);
    acc += probe10_2(&s2);
    acc += read10_2(&s2);
    acc += classify10_2(1, acc, acc);
    acc += accum10_2(5);
    acc += guard10_2(acc);
    S10_3 s3 = mk10_3(acc);
    bump10_3(&s3, 5);
    acc += probe10_3(&s3);
    acc += read10_3(&s3);
    acc += classify10_3(1, acc, acc);
    acc += accum10_3(5);
    acc += guard10_3(acc);
    return clampi(acc);
}
