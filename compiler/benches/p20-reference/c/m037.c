/* GENERATED C mirror of reference module m037. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S37_0;

static S37_0 mk37_0(long a) {
    S37_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe37_0(const S37_0 *s) {
    return s->a + s->n0;
}
static long read37_0(const S37_0 *s) {
    return s->a * 6;
}
static void bump37_0(S37_0 *s, long d) {
    s->a = s->a + d;
}
static long classify37_0(int tag, long a, long b) {
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
static long accum37_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard37_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S37_1;

static S37_1 mk37_1(long a) {
    S37_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe37_1(const S37_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read37_1(const S37_1 *s) {
    return s->a * 6;
}
static void bump37_1(S37_1 *s, long d) {
    s->a = s->a + d;
}
static long classify37_1(int tag, long a, long b) {
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
static long accum37_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard37_1(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S37_2;

static S37_2 mk37_2(long a) {
    S37_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe37_2(const S37_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read37_2(const S37_2 *s) {
    return s->a * 3;
}
static void bump37_2(S37_2 *s, long d) {
    s->a = s->a + d;
}
static long classify37_2(int tag, long a, long b) {
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
static long accum37_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard37_2(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S37_3;

static S37_3 mk37_3(long a) {
    S37_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe37_3(const S37_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read37_3(const S37_3 *s) {
    return s->a * 2;
}
static void bump37_3(S37_3 *s, long d) {
    s->a = s->a + d;
}
static long classify37_3(int tag, long a, long b) {
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
static long accum37_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard37_3(long x) {
    return x + 2;
}

long f037(long x) {
    long acc = x;
    acc += f008(x + 1);
    S37_0 s0 = mk37_0(acc);
    bump37_0(&s0, 4);
    acc += probe37_0(&s0);
    acc += read37_0(&s0);
    acc += classify37_0(1, acc, acc);
    acc += accum37_0(4);
    acc += guard37_0(acc);
    S37_1 s1 = mk37_1(acc);
    bump37_1(&s1, 3);
    acc += probe37_1(&s1);
    acc += read37_1(&s1);
    acc += classify37_1(1, acc, acc);
    acc += accum37_1(7);
    acc += guard37_1(acc);
    S37_2 s2 = mk37_2(acc);
    bump37_2(&s2, 6);
    acc += probe37_2(&s2);
    acc += read37_2(&s2);
    acc += classify37_2(1, acc, acc);
    acc += accum37_2(9);
    acc += guard37_2(acc);
    S37_3 s3 = mk37_3(acc);
    bump37_3(&s3, 7);
    acc += probe37_3(&s3);
    acc += read37_3(&s3);
    acc += classify37_3(1, acc, acc);
    acc += accum37_3(5);
    acc += guard37_3(acc);
    return clampi(acc);
}
