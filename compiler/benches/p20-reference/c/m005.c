/* GENERATED C mirror of reference module m005. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S5_0;

static S5_0 mk5_0(long a) {
    S5_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe5_0(const S5_0 *s) {
    return s->a + s->n0;
}
static long read5_0(const S5_0 *s) {
    return s->a * 6;
}
static void bump5_0(S5_0 *s, long d) {
    s->a = s->a + d;
}
static long classify5_0(int tag, long a, long b) {
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
static long accum5_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard5_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S5_1;

static S5_1 mk5_1(long a) {
    S5_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe5_1(const S5_1 *s) {
    return s->a + s->n0;
}
static long read5_1(const S5_1 *s) {
    return s->a * 7;
}
static void bump5_1(S5_1 *s, long d) {
    s->a = s->a + d;
}
static long classify5_1(int tag, long a, long b) {
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
static long accum5_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard5_1(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S5_2;

static S5_2 mk5_2(long a) {
    S5_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe5_2(const S5_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read5_2(const S5_2 *s) {
    return s->a * 4;
}
static void bump5_2(S5_2 *s, long d) {
    s->a = s->a + d;
}
static long classify5_2(int tag, long a, long b) {
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
static long accum5_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard5_2(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S5_3;

static S5_3 mk5_3(long a) {
    S5_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe5_3(const S5_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read5_3(const S5_3 *s) {
    return s->a * 6;
}
static void bump5_3(S5_3 *s, long d) {
    s->a = s->a + d;
}
static long classify5_3(int tag, long a, long b) {
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
static long accum5_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard5_3(long x) {
    return x + 8;
}

long f005(long x) {
    long acc = x;
    S5_0 s0 = mk5_0(acc);
    bump5_0(&s0, 2);
    acc += probe5_0(&s0);
    acc += read5_0(&s0);
    acc += classify5_0(1, acc, acc);
    acc += accum5_0(8);
    acc += guard5_0(acc);
    S5_1 s1 = mk5_1(acc);
    bump5_1(&s1, 8);
    acc += probe5_1(&s1);
    acc += read5_1(&s1);
    acc += classify5_1(1, acc, acc);
    acc += accum5_1(3);
    acc += guard5_1(acc);
    S5_2 s2 = mk5_2(acc);
    bump5_2(&s2, 9);
    acc += probe5_2(&s2);
    acc += read5_2(&s2);
    acc += classify5_2(1, acc, acc);
    acc += accum5_2(4);
    acc += guard5_2(acc);
    S5_3 s3 = mk5_3(acc);
    bump5_3(&s3, 7);
    acc += probe5_3(&s3);
    acc += read5_3(&s3);
    acc += classify5_3(1, acc, acc);
    acc += accum5_3(7);
    acc += guard5_3(acc);
    return clampi(acc);
}
