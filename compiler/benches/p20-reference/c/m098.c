/* GENERATED C mirror of reference module m098. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S98_0;

static S98_0 mk98_0(long a) {
    S98_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe98_0(const S98_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read98_0(const S98_0 *s) {
    return s->a * 7;
}
static void bump98_0(S98_0 *s, long d) {
    s->a = s->a + d;
}
static long classify98_0(int tag, long a, long b) {
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
static long accum98_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard98_0(long x) {
    return x + 1;
}

static long pick98_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S98_1;

static S98_1 mk98_1(long a) {
    S98_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe98_1(const S98_1 *s) {
    return s->a + s->n0;
}
static long read98_1(const S98_1 *s) {
    return s->a * 2;
}
static void bump98_1(S98_1 *s, long d) {
    s->a = s->a + d;
}
static long classify98_1(int tag, long a, long b) {
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
static long accum98_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard98_1(long x) {
    return x + 9;
}

static long pick98_1_0(long a, long b) { return a > b ? a : b; }
static long pick98_1_1(long a, long b) { return a > b ? a : b; }
static long pick98_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S98_2;

static S98_2 mk98_2(long a) {
    S98_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe98_2(const S98_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read98_2(const S98_2 *s) {
    return s->a * 6;
}
static void bump98_2(S98_2 *s, long d) {
    s->a = s->a + d;
}
static long classify98_2(int tag, long a, long b) {
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
static long accum98_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard98_2(long x) {
    return x + 9;
}

static long pick98_2_0(long a, long b) { return a > b ? a : b; }
static long pick98_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S98_3;

static S98_3 mk98_3(long a) {
    S98_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe98_3(const S98_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read98_3(const S98_3 *s) {
    return s->a * 5;
}
static void bump98_3(S98_3 *s, long d) {
    s->a = s->a + d;
}
static long classify98_3(int tag, long a, long b) {
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
static long accum98_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard98_3(long x) {
    return x + 2;
}

static long pick98_3_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S98_4;

static S98_4 mk98_4(long a) {
    S98_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe98_4(const S98_4 *s) {
    return s->a + s->n0;
}
static long read98_4(const S98_4 *s) {
    return s->a * 3;
}
static void bump98_4(S98_4 *s, long d) {
    s->a = s->a + d;
}
static long classify98_4(int tag, long a, long b) {
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
static long accum98_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard98_4(long x) {
    return x + 5;
}

static long pick98_4_0(long a, long b) { return a > b ? a : b; }
long f098(long x) {
    long acc = x;
    acc += f025(x + 1);
    acc += f037(x + 2);
    acc += f046(x + 3);
    S98_0 s0 = mk98_0(acc);
    bump98_0(&s0, 4);
    acc += probe98_0(&s0);
    acc += read98_0(&s0);
    acc += classify98_0(1, acc, acc);
    acc += accum98_0(5);
    acc += guard98_0(acc);
    acc += pick98_0_0(acc, acc + 9);
    S98_1 s1 = mk98_1(acc);
    bump98_1(&s1, 8);
    acc += probe98_1(&s1);
    acc += read98_1(&s1);
    acc += classify98_1(1, acc, acc);
    acc += accum98_1(6);
    acc += guard98_1(acc);
    acc += pick98_1_0(acc, acc + 6);
    acc += pick98_1_1(acc, acc + 2);
    acc += pick98_1_2(acc, acc + 7);
    S98_2 s2 = mk98_2(acc);
    bump98_2(&s2, 2);
    acc += probe98_2(&s2);
    acc += read98_2(&s2);
    acc += classify98_2(1, acc, acc);
    acc += accum98_2(3);
    acc += guard98_2(acc);
    acc += pick98_2_0(acc, acc + 9);
    acc += pick98_2_1(acc, acc + 5);
    S98_3 s3 = mk98_3(acc);
    bump98_3(&s3, 1);
    acc += probe98_3(&s3);
    acc += read98_3(&s3);
    acc += classify98_3(1, acc, acc);
    acc += accum98_3(4);
    acc += guard98_3(acc);
    acc += pick98_3_0(acc, acc + 6);
    S98_4 s4 = mk98_4(acc);
    bump98_4(&s4, 2);
    acc += probe98_4(&s4);
    acc += read98_4(&s4);
    acc += classify98_4(1, acc, acc);
    acc += accum98_4(3);
    acc += guard98_4(acc);
    acc += pick98_4_0(acc, acc + 1);
    return clampi(acc);
}
