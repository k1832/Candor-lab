/* GENERATED C mirror of reference module m044. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S44_0;

static S44_0 mk44_0(long a) {
    S44_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe44_0(const S44_0 *s) {
    return s->a + s->n0;
}
static long read44_0(const S44_0 *s) {
    return s->a * 2;
}
static void bump44_0(S44_0 *s, long d) {
    s->a = s->a + d;
}
static long classify44_0(int tag, long a, long b) {
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
static long accum44_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard44_0(long x) {
    return x + 8;
}

static long pick44_0_0(long a, long b) { return a > b ? a : b; }
static long pick44_0_1(long a, long b) { return a > b ? a : b; }
static long pick44_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S44_1;

static S44_1 mk44_1(long a) {
    S44_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe44_1(const S44_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read44_1(const S44_1 *s) {
    return s->a * 2;
}
static void bump44_1(S44_1 *s, long d) {
    s->a = s->a + d;
}
static long classify44_1(int tag, long a, long b) {
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
static long accum44_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard44_1(long x) {
    return x + 4;
}

static long pick44_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S44_2;

static S44_2 mk44_2(long a) {
    S44_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe44_2(const S44_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read44_2(const S44_2 *s) {
    return s->a * 2;
}
static void bump44_2(S44_2 *s, long d) {
    s->a = s->a + d;
}
static long classify44_2(int tag, long a, long b) {
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
static long accum44_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard44_2(long x) {
    return x + 8;
}

static long pick44_2_0(long a, long b) { return a > b ? a : b; }
static long pick44_2_1(long a, long b) { return a > b ? a : b; }
static long pick44_2_2(long a, long b) { return a > b ? a : b; }
long f044(long x) {
    long acc = x;
    acc += f016(x + 1);
    acc += f023(x + 2);
    S44_0 s0 = mk44_0(acc);
    bump44_0(&s0, 2);
    acc += probe44_0(&s0);
    acc += read44_0(&s0);
    acc += classify44_0(1, acc, acc);
    acc += accum44_0(6);
    acc += guard44_0(acc);
    acc += pick44_0_0(acc, acc + 3);
    acc += pick44_0_1(acc, acc + 9);
    acc += pick44_0_2(acc, acc + 8);
    S44_1 s1 = mk44_1(acc);
    bump44_1(&s1, 5);
    acc += probe44_1(&s1);
    acc += read44_1(&s1);
    acc += classify44_1(1, acc, acc);
    acc += accum44_1(7);
    acc += guard44_1(acc);
    acc += pick44_1_0(acc, acc + 5);
    S44_2 s2 = mk44_2(acc);
    bump44_2(&s2, 5);
    acc += probe44_2(&s2);
    acc += read44_2(&s2);
    acc += classify44_2(1, acc, acc);
    acc += accum44_2(5);
    acc += guard44_2(acc);
    acc += pick44_2_0(acc, acc + 1);
    acc += pick44_2_1(acc, acc + 4);
    acc += pick44_2_2(acc, acc + 3);
    return clampi(acc);
}
