/* GENERATED C mirror of reference module m089. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S89_0;

static S89_0 mk89_0(long a) {
    S89_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe89_0(const S89_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read89_0(const S89_0 *s) {
    return s->a * 3;
}
static void bump89_0(S89_0 *s, long d) {
    s->a = s->a + d;
}
static long classify89_0(int tag, long a, long b) {
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
static long accum89_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard89_0(long x) {
    return x + 4;
}

static long pick89_0_0(long a, long b) { return a > b ? a : b; }
static long pick89_0_1(long a, long b) { return a > b ? a : b; }
static long pick89_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S89_1;

static S89_1 mk89_1(long a) {
    S89_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe89_1(const S89_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read89_1(const S89_1 *s) {
    return s->a * 7;
}
static void bump89_1(S89_1 *s, long d) {
    s->a = s->a + d;
}
static long classify89_1(int tag, long a, long b) {
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
static long accum89_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard89_1(long x) {
    return x + 5;
}

static long pick89_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S89_2;

static S89_2 mk89_2(long a) {
    S89_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe89_2(const S89_2 *s) {
    return s->a + s->n0;
}
static long read89_2(const S89_2 *s) {
    return s->a * 3;
}
static void bump89_2(S89_2 *s, long d) {
    s->a = s->a + d;
}
static long classify89_2(int tag, long a, long b) {
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
static long accum89_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard89_2(long x) {
    return x + 7;
}

static long pick89_2_0(long a, long b) { return a > b ? a : b; }
static long pick89_2_1(long a, long b) { return a > b ? a : b; }
static long pick89_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S89_3;

static S89_3 mk89_3(long a) {
    S89_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe89_3(const S89_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read89_3(const S89_3 *s) {
    return s->a * 6;
}
static void bump89_3(S89_3 *s, long d) {
    s->a = s->a + d;
}
static long classify89_3(int tag, long a, long b) {
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
static long accum89_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard89_3(long x) {
    return x + 4;
}

static long pick89_3_0(long a, long b) { return a > b ? a : b; }
static long pick89_3_1(long a, long b) { return a > b ? a : b; }
static long pick89_3_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S89_4;

static S89_4 mk89_4(long a) {
    S89_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe89_4(const S89_4 *s) {
    return s->a + s->n0;
}
static long read89_4(const S89_4 *s) {
    return s->a * 3;
}
static void bump89_4(S89_4 *s, long d) {
    s->a = s->a + d;
}
static long classify89_4(int tag, long a, long b) {
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
static long accum89_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard89_4(long x) {
    return x + 9;
}

static long pick89_4_0(long a, long b) { return a > b ? a : b; }
static long pick89_4_1(long a, long b) { return a > b ? a : b; }
long f089(long x) {
    long acc = x;
    acc += f016(x + 1);
    acc += f057(x + 2);
    acc += f062(x + 3);
    acc += f072(x + 4);
    S89_0 s0 = mk89_0(acc);
    bump89_0(&s0, 2);
    acc += probe89_0(&s0);
    acc += read89_0(&s0);
    acc += classify89_0(1, acc, acc);
    acc += accum89_0(6);
    acc += guard89_0(acc);
    acc += pick89_0_0(acc, acc + 6);
    acc += pick89_0_1(acc, acc + 2);
    acc += pick89_0_2(acc, acc + 9);
    S89_1 s1 = mk89_1(acc);
    bump89_1(&s1, 4);
    acc += probe89_1(&s1);
    acc += read89_1(&s1);
    acc += classify89_1(1, acc, acc);
    acc += accum89_1(8);
    acc += guard89_1(acc);
    acc += pick89_1_0(acc, acc + 4);
    S89_2 s2 = mk89_2(acc);
    bump89_2(&s2, 7);
    acc += probe89_2(&s2);
    acc += read89_2(&s2);
    acc += classify89_2(1, acc, acc);
    acc += accum89_2(9);
    acc += guard89_2(acc);
    acc += pick89_2_0(acc, acc + 2);
    acc += pick89_2_1(acc, acc + 2);
    acc += pick89_2_2(acc, acc + 8);
    S89_3 s3 = mk89_3(acc);
    bump89_3(&s3, 4);
    acc += probe89_3(&s3);
    acc += read89_3(&s3);
    acc += classify89_3(1, acc, acc);
    acc += accum89_3(4);
    acc += guard89_3(acc);
    acc += pick89_3_0(acc, acc + 5);
    acc += pick89_3_1(acc, acc + 1);
    acc += pick89_3_2(acc, acc + 4);
    S89_4 s4 = mk89_4(acc);
    bump89_4(&s4, 1);
    acc += probe89_4(&s4);
    acc += read89_4(&s4);
    acc += classify89_4(1, acc, acc);
    acc += accum89_4(4);
    acc += guard89_4(acc);
    acc += pick89_4_0(acc, acc + 1);
    acc += pick89_4_1(acc, acc + 9);
    return clampi(acc);
}
