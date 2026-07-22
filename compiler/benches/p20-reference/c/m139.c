/* GENERATED C mirror of reference module m139. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S139_0;

static S139_0 mk139_0(long a) {
    S139_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe139_0(const S139_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read139_0(const S139_0 *s) {
    return s->a * 3;
}
static void bump139_0(S139_0 *s, long d) {
    s->a = s->a + d;
}
static long classify139_0(int tag, long a, long b) {
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
static long accum139_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard139_0(long x) {
    return x + 3;
}

static long pick139_0_0(long a, long b) { return a > b ? a : b; }
static long pick139_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S139_1;

static S139_1 mk139_1(long a) {
    S139_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe139_1(const S139_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read139_1(const S139_1 *s) {
    return s->a * 7;
}
static void bump139_1(S139_1 *s, long d) {
    s->a = s->a + d;
}
static long classify139_1(int tag, long a, long b) {
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
static long accum139_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard139_1(long x) {
    return x + 2;
}

static long pick139_1_0(long a, long b) { return a > b ? a : b; }
static long pick139_1_1(long a, long b) { return a > b ? a : b; }
static long pick139_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S139_2;

static S139_2 mk139_2(long a) {
    S139_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe139_2(const S139_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read139_2(const S139_2 *s) {
    return s->a * 6;
}
static void bump139_2(S139_2 *s, long d) {
    s->a = s->a + d;
}
static long classify139_2(int tag, long a, long b) {
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
static long accum139_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard139_2(long x) {
    return x + 3;
}

static long pick139_2_0(long a, long b) { return a > b ? a : b; }
static long pick139_2_1(long a, long b) { return a > b ? a : b; }
static long pick139_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S139_3;

static S139_3 mk139_3(long a) {
    S139_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe139_3(const S139_3 *s) {
    return s->a + s->n0;
}
static long read139_3(const S139_3 *s) {
    return s->a * 3;
}
static void bump139_3(S139_3 *s, long d) {
    s->a = s->a + d;
}
static long classify139_3(int tag, long a, long b) {
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
static long accum139_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard139_3(long x) {
    return x + 1;
}

static long pick139_3_0(long a, long b) { return a > b ? a : b; }
long f139(long x) {
    long acc = x;
    acc += f101(x + 1);
    acc += f110(x + 2);
    S139_0 s0 = mk139_0(acc);
    bump139_0(&s0, 7);
    acc += probe139_0(&s0);
    acc += read139_0(&s0);
    acc += classify139_0(1, acc, acc);
    acc += accum139_0(8);
    acc += guard139_0(acc);
    acc += pick139_0_0(acc, acc + 3);
    acc += pick139_0_1(acc, acc + 1);
    S139_1 s1 = mk139_1(acc);
    bump139_1(&s1, 8);
    acc += probe139_1(&s1);
    acc += read139_1(&s1);
    acc += classify139_1(1, acc, acc);
    acc += accum139_1(8);
    acc += guard139_1(acc);
    acc += pick139_1_0(acc, acc + 9);
    acc += pick139_1_1(acc, acc + 3);
    acc += pick139_1_2(acc, acc + 5);
    S139_2 s2 = mk139_2(acc);
    bump139_2(&s2, 6);
    acc += probe139_2(&s2);
    acc += read139_2(&s2);
    acc += classify139_2(1, acc, acc);
    acc += accum139_2(4);
    acc += guard139_2(acc);
    acc += pick139_2_0(acc, acc + 9);
    acc += pick139_2_1(acc, acc + 8);
    acc += pick139_2_2(acc, acc + 8);
    S139_3 s3 = mk139_3(acc);
    bump139_3(&s3, 1);
    acc += probe139_3(&s3);
    acc += read139_3(&s3);
    acc += classify139_3(1, acc, acc);
    acc += accum139_3(6);
    acc += guard139_3(acc);
    acc += pick139_3_0(acc, acc + 3);
    return clampi(acc);
}
