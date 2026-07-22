/* GENERATED C mirror of reference module m097. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S97_0;

static S97_0 mk97_0(long a) {
    S97_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe97_0(const S97_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read97_0(const S97_0 *s) {
    return s->a * 2;
}
static void bump97_0(S97_0 *s, long d) {
    s->a = s->a + d;
}
static long classify97_0(int tag, long a, long b) {
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
static long accum97_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard97_0(long x) {
    return x + 1;
}

static long pick97_0_0(long a, long b) { return a > b ? a : b; }
static long pick97_0_1(long a, long b) { return a > b ? a : b; }
static long pick97_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S97_1;

static S97_1 mk97_1(long a) {
    S97_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe97_1(const S97_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read97_1(const S97_1 *s) {
    return s->a * 4;
}
static void bump97_1(S97_1 *s, long d) {
    s->a = s->a + d;
}
static long classify97_1(int tag, long a, long b) {
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
static long accum97_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard97_1(long x) {
    return x + 8;
}

static long pick97_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S97_2;

static S97_2 mk97_2(long a) {
    S97_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe97_2(const S97_2 *s) {
    return s->a + s->n0;
}
static long read97_2(const S97_2 *s) {
    return s->a * 5;
}
static void bump97_2(S97_2 *s, long d) {
    s->a = s->a + d;
}
static long classify97_2(int tag, long a, long b) {
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
static long accum97_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard97_2(long x) {
    return x + 2;
}

static long pick97_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S97_3;

static S97_3 mk97_3(long a) {
    S97_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe97_3(const S97_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read97_3(const S97_3 *s) {
    return s->a * 2;
}
static void bump97_3(S97_3 *s, long d) {
    s->a = s->a + d;
}
static long classify97_3(int tag, long a, long b) {
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
static long accum97_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard97_3(long x) {
    return x + 6;
}

static long pick97_3_0(long a, long b) { return a > b ? a : b; }
static long pick97_3_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S97_4;

static S97_4 mk97_4(long a) {
    S97_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe97_4(const S97_4 *s) {
    return s->a + s->n0;
}
static long read97_4(const S97_4 *s) {
    return s->a * 5;
}
static void bump97_4(S97_4 *s, long d) {
    s->a = s->a + d;
}
static long classify97_4(int tag, long a, long b) {
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
static long accum97_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard97_4(long x) {
    return x + 9;
}

static long pick97_4_0(long a, long b) { return a > b ? a : b; }
static long pick97_4_1(long a, long b) { return a > b ? a : b; }
static long pick97_4_2(long a, long b) { return a > b ? a : b; }
long f097(long x) {
    long acc = x;
    acc += f017(x + 1);
    acc += f074(x + 2);
    S97_0 s0 = mk97_0(acc);
    bump97_0(&s0, 1);
    acc += probe97_0(&s0);
    acc += read97_0(&s0);
    acc += classify97_0(1, acc, acc);
    acc += accum97_0(6);
    acc += guard97_0(acc);
    acc += pick97_0_0(acc, acc + 1);
    acc += pick97_0_1(acc, acc + 1);
    acc += pick97_0_2(acc, acc + 8);
    S97_1 s1 = mk97_1(acc);
    bump97_1(&s1, 6);
    acc += probe97_1(&s1);
    acc += read97_1(&s1);
    acc += classify97_1(1, acc, acc);
    acc += accum97_1(5);
    acc += guard97_1(acc);
    acc += pick97_1_0(acc, acc + 8);
    S97_2 s2 = mk97_2(acc);
    bump97_2(&s2, 9);
    acc += probe97_2(&s2);
    acc += read97_2(&s2);
    acc += classify97_2(1, acc, acc);
    acc += accum97_2(5);
    acc += guard97_2(acc);
    acc += pick97_2_0(acc, acc + 1);
    S97_3 s3 = mk97_3(acc);
    bump97_3(&s3, 5);
    acc += probe97_3(&s3);
    acc += read97_3(&s3);
    acc += classify97_3(1, acc, acc);
    acc += accum97_3(4);
    acc += guard97_3(acc);
    acc += pick97_3_0(acc, acc + 8);
    acc += pick97_3_1(acc, acc + 6);
    S97_4 s4 = mk97_4(acc);
    bump97_4(&s4, 4);
    acc += probe97_4(&s4);
    acc += read97_4(&s4);
    acc += classify97_4(1, acc, acc);
    acc += accum97_4(6);
    acc += guard97_4(acc);
    acc += pick97_4_0(acc, acc + 6);
    acc += pick97_4_1(acc, acc + 6);
    acc += pick97_4_2(acc, acc + 6);
    return clampi(acc);
}
