/* GENERATED C mirror of reference module m007. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S7_0;

static S7_0 mk7_0(long a) {
    S7_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe7_0(const S7_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read7_0(const S7_0 *s) {
    return s->a * 5;
}
static void bump7_0(S7_0 *s, long d) {
    s->a = s->a + d;
}
static long classify7_0(int tag, long a, long b) {
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
static long accum7_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard7_0(long x) {
    return x + 6;
}

static long pick7_0_0(long a, long b) { return a > b ? a : b; }
static long pick7_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S7_1;

static S7_1 mk7_1(long a) {
    S7_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe7_1(const S7_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read7_1(const S7_1 *s) {
    return s->a * 6;
}
static void bump7_1(S7_1 *s, long d) {
    s->a = s->a + d;
}
static long classify7_1(int tag, long a, long b) {
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
static long accum7_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard7_1(long x) {
    return x + 9;
}

static long pick7_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S7_2;

static S7_2 mk7_2(long a) {
    S7_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe7_2(const S7_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read7_2(const S7_2 *s) {
    return s->a * 3;
}
static void bump7_2(S7_2 *s, long d) {
    s->a = s->a + d;
}
static long classify7_2(int tag, long a, long b) {
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
static long accum7_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard7_2(long x) {
    return x + 3;
}

static long pick7_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S7_3;

static S7_3 mk7_3(long a) {
    S7_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe7_3(const S7_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read7_3(const S7_3 *s) {
    return s->a * 6;
}
static void bump7_3(S7_3 *s, long d) {
    s->a = s->a + d;
}
static long classify7_3(int tag, long a, long b) {
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
static long accum7_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard7_3(long x) {
    return x + 3;
}

static long pick7_3_0(long a, long b) { return a > b ? a : b; }
static long pick7_3_1(long a, long b) { return a > b ? a : b; }
static long pick7_3_2(long a, long b) { return a > b ? a : b; }
long f007(long x) {
    long acc = x;
    S7_0 s0 = mk7_0(acc);
    bump7_0(&s0, 2);
    acc += probe7_0(&s0);
    acc += read7_0(&s0);
    acc += classify7_0(1, acc, acc);
    acc += accum7_0(3);
    acc += guard7_0(acc);
    acc += pick7_0_0(acc, acc + 7);
    acc += pick7_0_1(acc, acc + 5);
    S7_1 s1 = mk7_1(acc);
    bump7_1(&s1, 4);
    acc += probe7_1(&s1);
    acc += read7_1(&s1);
    acc += classify7_1(1, acc, acc);
    acc += accum7_1(3);
    acc += guard7_1(acc);
    acc += pick7_1_0(acc, acc + 7);
    S7_2 s2 = mk7_2(acc);
    bump7_2(&s2, 4);
    acc += probe7_2(&s2);
    acc += read7_2(&s2);
    acc += classify7_2(1, acc, acc);
    acc += accum7_2(8);
    acc += guard7_2(acc);
    acc += pick7_2_0(acc, acc + 8);
    S7_3 s3 = mk7_3(acc);
    bump7_3(&s3, 9);
    acc += probe7_3(&s3);
    acc += read7_3(&s3);
    acc += classify7_3(1, acc, acc);
    acc += accum7_3(9);
    acc += guard7_3(acc);
    acc += pick7_3_0(acc, acc + 4);
    acc += pick7_3_1(acc, acc + 1);
    acc += pick7_3_2(acc, acc + 7);
    return clampi(acc);
}
