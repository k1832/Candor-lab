/* GENERATED C mirror of reference module m061. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S61_0;

static S61_0 mk61_0(long a) {
    S61_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe61_0(const S61_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read61_0(const S61_0 *s) {
    return s->a * 2;
}
static void bump61_0(S61_0 *s, long d) {
    s->a = s->a + d;
}
static long classify61_0(int tag, long a, long b) {
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
static long accum61_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard61_0(long x) {
    return x + 3;
}

static long pick61_0_0(long a, long b) { return a > b ? a : b; }
static long pick61_0_1(long a, long b) { return a > b ? a : b; }
static long pick61_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S61_1;

static S61_1 mk61_1(long a) {
    S61_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe61_1(const S61_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read61_1(const S61_1 *s) {
    return s->a * 5;
}
static void bump61_1(S61_1 *s, long d) {
    s->a = s->a + d;
}
static long classify61_1(int tag, long a, long b) {
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
static long accum61_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard61_1(long x) {
    return x + 4;
}

static long pick61_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S61_2;

static S61_2 mk61_2(long a) {
    S61_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe61_2(const S61_2 *s) {
    return s->a + s->n0;
}
static long read61_2(const S61_2 *s) {
    return s->a * 7;
}
static void bump61_2(S61_2 *s, long d) {
    s->a = s->a + d;
}
static long classify61_2(int tag, long a, long b) {
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
static long accum61_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard61_2(long x) {
    return x + 4;
}

static long pick61_2_0(long a, long b) { return a > b ? a : b; }
static long pick61_2_1(long a, long b) { return a > b ? a : b; }
static long pick61_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S61_3;

static S61_3 mk61_3(long a) {
    S61_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe61_3(const S61_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read61_3(const S61_3 *s) {
    return s->a * 4;
}
static void bump61_3(S61_3 *s, long d) {
    s->a = s->a + d;
}
static long classify61_3(int tag, long a, long b) {
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
static long accum61_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard61_3(long x) {
    return x + 5;
}

static long pick61_3_0(long a, long b) { return a > b ? a : b; }
long f061(long x) {
    long acc = x;
    acc += f008(x + 1);
    acc += f009(x + 2);
    acc += f024(x + 3);
    acc += f033(x + 4);
    S61_0 s0 = mk61_0(acc);
    bump61_0(&s0, 3);
    acc += probe61_0(&s0);
    acc += read61_0(&s0);
    acc += classify61_0(1, acc, acc);
    acc += accum61_0(8);
    acc += guard61_0(acc);
    acc += pick61_0_0(acc, acc + 8);
    acc += pick61_0_1(acc, acc + 1);
    acc += pick61_0_2(acc, acc + 7);
    S61_1 s1 = mk61_1(acc);
    bump61_1(&s1, 5);
    acc += probe61_1(&s1);
    acc += read61_1(&s1);
    acc += classify61_1(1, acc, acc);
    acc += accum61_1(7);
    acc += guard61_1(acc);
    acc += pick61_1_0(acc, acc + 1);
    S61_2 s2 = mk61_2(acc);
    bump61_2(&s2, 4);
    acc += probe61_2(&s2);
    acc += read61_2(&s2);
    acc += classify61_2(1, acc, acc);
    acc += accum61_2(3);
    acc += guard61_2(acc);
    acc += pick61_2_0(acc, acc + 7);
    acc += pick61_2_1(acc, acc + 2);
    acc += pick61_2_2(acc, acc + 9);
    S61_3 s3 = mk61_3(acc);
    bump61_3(&s3, 1);
    acc += probe61_3(&s3);
    acc += read61_3(&s3);
    acc += classify61_3(1, acc, acc);
    acc += accum61_3(5);
    acc += guard61_3(acc);
    acc += pick61_3_0(acc, acc + 3);
    return clampi(acc);
}
