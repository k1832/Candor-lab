/* GENERATED C mirror of reference module m006. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S6_0;

static S6_0 mk6_0(long a) {
    S6_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe6_0(const S6_0 *s) {
    return s->a + s->n0;
}
static long read6_0(const S6_0 *s) {
    return s->a * 7;
}
static void bump6_0(S6_0 *s, long d) {
    s->a = s->a + d;
}
static long classify6_0(int tag, long a, long b) {
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
static long accum6_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard6_0(long x) {
    return x + 4;
}

static long pick6_0_0(long a, long b) { return a > b ? a : b; }
static long pick6_0_1(long a, long b) { return a > b ? a : b; }
static long pick6_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S6_1;

static S6_1 mk6_1(long a) {
    S6_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe6_1(const S6_1 *s) {
    return s->a + s->n0;
}
static long read6_1(const S6_1 *s) {
    return s->a * 2;
}
static void bump6_1(S6_1 *s, long d) {
    s->a = s->a + d;
}
static long classify6_1(int tag, long a, long b) {
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
static long accum6_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard6_1(long x) {
    return x + 7;
}

static long pick6_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S6_2;

static S6_2 mk6_2(long a) {
    S6_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe6_2(const S6_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read6_2(const S6_2 *s) {
    return s->a * 2;
}
static void bump6_2(S6_2 *s, long d) {
    s->a = s->a + d;
}
static long classify6_2(int tag, long a, long b) {
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
static long accum6_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard6_2(long x) {
    return x + 7;
}

static long pick6_2_0(long a, long b) { return a > b ? a : b; }
static long pick6_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S6_3;

static S6_3 mk6_3(long a) {
    S6_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe6_3(const S6_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read6_3(const S6_3 *s) {
    return s->a * 2;
}
static void bump6_3(S6_3 *s, long d) {
    s->a = s->a + d;
}
static long classify6_3(int tag, long a, long b) {
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
static long accum6_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard6_3(long x) {
    return x + 9;
}

static long pick6_3_0(long a, long b) { return a > b ? a : b; }
static long pick6_3_1(long a, long b) { return a > b ? a : b; }
static long pick6_3_2(long a, long b) { return a > b ? a : b; }
long f006(long x) {
    long acc = x;
    S6_0 s0 = mk6_0(acc);
    bump6_0(&s0, 5);
    acc += probe6_0(&s0);
    acc += read6_0(&s0);
    acc += classify6_0(1, acc, acc);
    acc += accum6_0(4);
    acc += guard6_0(acc);
    acc += pick6_0_0(acc, acc + 8);
    acc += pick6_0_1(acc, acc + 2);
    acc += pick6_0_2(acc, acc + 5);
    S6_1 s1 = mk6_1(acc);
    bump6_1(&s1, 8);
    acc += probe6_1(&s1);
    acc += read6_1(&s1);
    acc += classify6_1(1, acc, acc);
    acc += accum6_1(3);
    acc += guard6_1(acc);
    acc += pick6_1_0(acc, acc + 7);
    S6_2 s2 = mk6_2(acc);
    bump6_2(&s2, 9);
    acc += probe6_2(&s2);
    acc += read6_2(&s2);
    acc += classify6_2(1, acc, acc);
    acc += accum6_2(8);
    acc += guard6_2(acc);
    acc += pick6_2_0(acc, acc + 3);
    acc += pick6_2_1(acc, acc + 1);
    S6_3 s3 = mk6_3(acc);
    bump6_3(&s3, 5);
    acc += probe6_3(&s3);
    acc += read6_3(&s3);
    acc += classify6_3(1, acc, acc);
    acc += accum6_3(9);
    acc += guard6_3(acc);
    acc += pick6_3_0(acc, acc + 5);
    acc += pick6_3_1(acc, acc + 2);
    acc += pick6_3_2(acc, acc + 5);
    return clampi(acc);
}
