/* GENERATED C mirror of reference module m133. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S133_0;

static S133_0 mk133_0(long a) {
    S133_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe133_0(const S133_0 *s) {
    return s->a + s->n0;
}
static long read133_0(const S133_0 *s) {
    return s->a * 4;
}
static void bump133_0(S133_0 *s, long d) {
    s->a = s->a + d;
}
static long classify133_0(int tag, long a, long b) {
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
static long accum133_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard133_0(long x) {
    return x + 3;
}

static long pick133_0_0(long a, long b) { return a > b ? a : b; }
static long pick133_0_1(long a, long b) { return a > b ? a : b; }
static long pick133_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S133_1;

static S133_1 mk133_1(long a) {
    S133_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe133_1(const S133_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read133_1(const S133_1 *s) {
    return s->a * 4;
}
static void bump133_1(S133_1 *s, long d) {
    s->a = s->a + d;
}
static long classify133_1(int tag, long a, long b) {
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
static long accum133_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard133_1(long x) {
    return x + 1;
}

static long pick133_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S133_2;

static S133_2 mk133_2(long a) {
    S133_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe133_2(const S133_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read133_2(const S133_2 *s) {
    return s->a * 7;
}
static void bump133_2(S133_2 *s, long d) {
    s->a = s->a + d;
}
static long classify133_2(int tag, long a, long b) {
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
static long accum133_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard133_2(long x) {
    return x + 6;
}

static long pick133_2_0(long a, long b) { return a > b ? a : b; }
static long pick133_2_1(long a, long b) { return a > b ? a : b; }
static long pick133_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S133_3;

static S133_3 mk133_3(long a) {
    S133_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe133_3(const S133_3 *s) {
    return s->a + s->n0;
}
static long read133_3(const S133_3 *s) {
    return s->a * 4;
}
static void bump133_3(S133_3 *s, long d) {
    s->a = s->a + d;
}
static long classify133_3(int tag, long a, long b) {
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
static long accum133_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard133_3(long x) {
    return x + 5;
}

static long pick133_3_0(long a, long b) { return a > b ? a : b; }
static long pick133_3_1(long a, long b) { return a > b ? a : b; }
long f133(long x) {
    long acc = x;
    acc += f020(x + 1);
    acc += f085(x + 2);
    acc += f096(x + 3);
    acc += f104(x + 4);
    S133_0 s0 = mk133_0(acc);
    bump133_0(&s0, 4);
    acc += probe133_0(&s0);
    acc += read133_0(&s0);
    acc += classify133_0(1, acc, acc);
    acc += accum133_0(4);
    acc += guard133_0(acc);
    acc += pick133_0_0(acc, acc + 3);
    acc += pick133_0_1(acc, acc + 4);
    acc += pick133_0_2(acc, acc + 4);
    S133_1 s1 = mk133_1(acc);
    bump133_1(&s1, 3);
    acc += probe133_1(&s1);
    acc += read133_1(&s1);
    acc += classify133_1(1, acc, acc);
    acc += accum133_1(9);
    acc += guard133_1(acc);
    acc += pick133_1_0(acc, acc + 7);
    S133_2 s2 = mk133_2(acc);
    bump133_2(&s2, 9);
    acc += probe133_2(&s2);
    acc += read133_2(&s2);
    acc += classify133_2(1, acc, acc);
    acc += accum133_2(6);
    acc += guard133_2(acc);
    acc += pick133_2_0(acc, acc + 5);
    acc += pick133_2_1(acc, acc + 9);
    acc += pick133_2_2(acc, acc + 1);
    S133_3 s3 = mk133_3(acc);
    bump133_3(&s3, 2);
    acc += probe133_3(&s3);
    acc += read133_3(&s3);
    acc += classify133_3(1, acc, acc);
    acc += accum133_3(4);
    acc += guard133_3(acc);
    acc += pick133_3_0(acc, acc + 4);
    acc += pick133_3_1(acc, acc + 7);
    return clampi(acc);
}
