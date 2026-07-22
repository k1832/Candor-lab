/* GENERATED C mirror of reference module m078. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S78_0;

static S78_0 mk78_0(long a) {
    S78_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe78_0(const S78_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read78_0(const S78_0 *s) {
    return s->a * 5;
}
static void bump78_0(S78_0 *s, long d) {
    s->a = s->a + d;
}
static long classify78_0(int tag, long a, long b) {
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
static long accum78_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard78_0(long x) {
    return x + 3;
}

static long pick78_0_0(long a, long b) { return a > b ? a : b; }
static long pick78_0_1(long a, long b) { return a > b ? a : b; }
static long pick78_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S78_1;

static S78_1 mk78_1(long a) {
    S78_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe78_1(const S78_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read78_1(const S78_1 *s) {
    return s->a * 2;
}
static void bump78_1(S78_1 *s, long d) {
    s->a = s->a + d;
}
static long classify78_1(int tag, long a, long b) {
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
static long accum78_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard78_1(long x) {
    return x + 4;
}

static long pick78_1_0(long a, long b) { return a > b ? a : b; }
static long pick78_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S78_2;

static S78_2 mk78_2(long a) {
    S78_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe78_2(const S78_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read78_2(const S78_2 *s) {
    return s->a * 5;
}
static void bump78_2(S78_2 *s, long d) {
    s->a = s->a + d;
}
static long classify78_2(int tag, long a, long b) {
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
static long accum78_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard78_2(long x) {
    return x + 4;
}

static long pick78_2_0(long a, long b) { return a > b ? a : b; }
static long pick78_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S78_3;

static S78_3 mk78_3(long a) {
    S78_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe78_3(const S78_3 *s) {
    return s->a + s->n0;
}
static long read78_3(const S78_3 *s) {
    return s->a * 6;
}
static void bump78_3(S78_3 *s, long d) {
    s->a = s->a + d;
}
static long classify78_3(int tag, long a, long b) {
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
static long accum78_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard78_3(long x) {
    return x + 7;
}

static long pick78_3_0(long a, long b) { return a > b ? a : b; }
long f078(long x) {
    long acc = x;
    acc += f047(x + 1);
    acc += f064(x + 2);
    acc += f066(x + 3);
    S78_0 s0 = mk78_0(acc);
    bump78_0(&s0, 8);
    acc += probe78_0(&s0);
    acc += read78_0(&s0);
    acc += classify78_0(1, acc, acc);
    acc += accum78_0(5);
    acc += guard78_0(acc);
    acc += pick78_0_0(acc, acc + 7);
    acc += pick78_0_1(acc, acc + 6);
    acc += pick78_0_2(acc, acc + 3);
    S78_1 s1 = mk78_1(acc);
    bump78_1(&s1, 2);
    acc += probe78_1(&s1);
    acc += read78_1(&s1);
    acc += classify78_1(1, acc, acc);
    acc += accum78_1(9);
    acc += guard78_1(acc);
    acc += pick78_1_0(acc, acc + 5);
    acc += pick78_1_1(acc, acc + 6);
    S78_2 s2 = mk78_2(acc);
    bump78_2(&s2, 8);
    acc += probe78_2(&s2);
    acc += read78_2(&s2);
    acc += classify78_2(1, acc, acc);
    acc += accum78_2(9);
    acc += guard78_2(acc);
    acc += pick78_2_0(acc, acc + 7);
    acc += pick78_2_1(acc, acc + 3);
    S78_3 s3 = mk78_3(acc);
    bump78_3(&s3, 3);
    acc += probe78_3(&s3);
    acc += read78_3(&s3);
    acc += classify78_3(1, acc, acc);
    acc += accum78_3(5);
    acc += guard78_3(acc);
    acc += pick78_3_0(acc, acc + 6);
    return clampi(acc);
}
