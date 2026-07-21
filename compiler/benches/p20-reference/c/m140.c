/* GENERATED C mirror of reference module m140. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S140_0;

static S140_0 mk140_0(long a) {
    S140_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe140_0(const S140_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read140_0(const S140_0 *s) {
    return s->a * 7;
}
static void bump140_0(S140_0 *s, long d) {
    s->a = s->a + d;
}
static long classify140_0(int tag, long a, long b) {
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
static long accum140_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard140_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S140_1;

static S140_1 mk140_1(long a) {
    S140_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe140_1(const S140_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read140_1(const S140_1 *s) {
    return s->a * 2;
}
static void bump140_1(S140_1 *s, long d) {
    s->a = s->a + d;
}
static long classify140_1(int tag, long a, long b) {
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
static long accum140_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard140_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S140_2;

static S140_2 mk140_2(long a) {
    S140_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe140_2(const S140_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read140_2(const S140_2 *s) {
    return s->a * 5;
}
static void bump140_2(S140_2 *s, long d) {
    s->a = s->a + d;
}
static long classify140_2(int tag, long a, long b) {
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
static long accum140_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard140_2(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S140_3;

static S140_3 mk140_3(long a) {
    S140_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe140_3(const S140_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read140_3(const S140_3 *s) {
    return s->a * 6;
}
static void bump140_3(S140_3 *s, long d) {
    s->a = s->a + d;
}
static long classify140_3(int tag, long a, long b) {
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
static long accum140_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard140_3(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S140_4;

static S140_4 mk140_4(long a) {
    S140_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe140_4(const S140_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read140_4(const S140_4 *s) {
    return s->a * 4;
}
static void bump140_4(S140_4 *s, long d) {
    s->a = s->a + d;
}
static long classify140_4(int tag, long a, long b) {
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
static long accum140_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard140_4(long x) {
    return x + 8;
}

long f140(long x) {
    long acc = x;
    acc += f014(x + 1);
    acc += f019(x + 2);
    acc += f066(x + 3);
    acc += f111(x + 4);
    S140_0 s0 = mk140_0(acc);
    bump140_0(&s0, 5);
    acc += probe140_0(&s0);
    acc += read140_0(&s0);
    acc += classify140_0(1, acc, acc);
    acc += accum140_0(9);
    acc += guard140_0(acc);
    S140_1 s1 = mk140_1(acc);
    bump140_1(&s1, 8);
    acc += probe140_1(&s1);
    acc += read140_1(&s1);
    acc += classify140_1(1, acc, acc);
    acc += accum140_1(6);
    acc += guard140_1(acc);
    S140_2 s2 = mk140_2(acc);
    bump140_2(&s2, 1);
    acc += probe140_2(&s2);
    acc += read140_2(&s2);
    acc += classify140_2(1, acc, acc);
    acc += accum140_2(9);
    acc += guard140_2(acc);
    S140_3 s3 = mk140_3(acc);
    bump140_3(&s3, 9);
    acc += probe140_3(&s3);
    acc += read140_3(&s3);
    acc += classify140_3(1, acc, acc);
    acc += accum140_3(3);
    acc += guard140_3(acc);
    S140_4 s4 = mk140_4(acc);
    bump140_4(&s4, 9);
    acc += probe140_4(&s4);
    acc += read140_4(&s4);
    acc += classify140_4(1, acc, acc);
    acc += accum140_4(9);
    acc += guard140_4(acc);
    return clampi(acc);
}
