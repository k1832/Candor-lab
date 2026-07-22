/* GENERATED C mirror of reference module m156. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S156_0;

static S156_0 mk156_0(long a) {
    S156_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe156_0(const S156_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read156_0(const S156_0 *s) {
    return s->a * 5;
}
static void bump156_0(S156_0 *s, long d) {
    s->a = s->a + d;
}
static long classify156_0(int tag, long a, long b) {
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
static long accum156_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard156_0(long x) {
    return x + 1;
}

static long pick156_0_0(long a, long b) { return a > b ? a : b; }
static long pick156_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S156_1;

static S156_1 mk156_1(long a) {
    S156_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe156_1(const S156_1 *s) {
    return s->a + s->n0;
}
static long read156_1(const S156_1 *s) {
    return s->a * 5;
}
static void bump156_1(S156_1 *s, long d) {
    s->a = s->a + d;
}
static long classify156_1(int tag, long a, long b) {
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
static long accum156_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard156_1(long x) {
    return x + 9;
}

static long pick156_1_0(long a, long b) { return a > b ? a : b; }
static long pick156_1_1(long a, long b) { return a > b ? a : b; }
static long pick156_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S156_2;

static S156_2 mk156_2(long a) {
    S156_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe156_2(const S156_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read156_2(const S156_2 *s) {
    return s->a * 7;
}
static void bump156_2(S156_2 *s, long d) {
    s->a = s->a + d;
}
static long classify156_2(int tag, long a, long b) {
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
static long accum156_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard156_2(long x) {
    return x + 7;
}

static long pick156_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S156_3;

static S156_3 mk156_3(long a) {
    S156_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe156_3(const S156_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read156_3(const S156_3 *s) {
    return s->a * 5;
}
static void bump156_3(S156_3 *s, long d) {
    s->a = s->a + d;
}
static long classify156_3(int tag, long a, long b) {
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
static long accum156_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard156_3(long x) {
    return x + 9;
}

static long pick156_3_0(long a, long b) { return a > b ? a : b; }
static long pick156_3_1(long a, long b) { return a > b ? a : b; }
static long pick156_3_2(long a, long b) { return a > b ? a : b; }
long f156(long x) {
    long acc = x;
    acc += f058(x + 1);
    acc += f073(x + 2);
    S156_0 s0 = mk156_0(acc);
    bump156_0(&s0, 1);
    acc += probe156_0(&s0);
    acc += read156_0(&s0);
    acc += classify156_0(1, acc, acc);
    acc += accum156_0(5);
    acc += guard156_0(acc);
    acc += pick156_0_0(acc, acc + 7);
    acc += pick156_0_1(acc, acc + 3);
    S156_1 s1 = mk156_1(acc);
    bump156_1(&s1, 9);
    acc += probe156_1(&s1);
    acc += read156_1(&s1);
    acc += classify156_1(1, acc, acc);
    acc += accum156_1(7);
    acc += guard156_1(acc);
    acc += pick156_1_0(acc, acc + 4);
    acc += pick156_1_1(acc, acc + 6);
    acc += pick156_1_2(acc, acc + 1);
    S156_2 s2 = mk156_2(acc);
    bump156_2(&s2, 6);
    acc += probe156_2(&s2);
    acc += read156_2(&s2);
    acc += classify156_2(1, acc, acc);
    acc += accum156_2(6);
    acc += guard156_2(acc);
    acc += pick156_2_0(acc, acc + 9);
    S156_3 s3 = mk156_3(acc);
    bump156_3(&s3, 4);
    acc += probe156_3(&s3);
    acc += read156_3(&s3);
    acc += classify156_3(1, acc, acc);
    acc += accum156_3(8);
    acc += guard156_3(acc);
    acc += pick156_3_0(acc, acc + 9);
    acc += pick156_3_1(acc, acc + 7);
    acc += pick156_3_2(acc, acc + 3);
    return clampi(acc);
}
