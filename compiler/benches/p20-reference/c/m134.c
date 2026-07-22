/* GENERATED C mirror of reference module m134. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S134_0;

static S134_0 mk134_0(long a) {
    S134_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe134_0(const S134_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read134_0(const S134_0 *s) {
    return s->a * 2;
}
static void bump134_0(S134_0 *s, long d) {
    s->a = s->a + d;
}
static long classify134_0(int tag, long a, long b) {
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
static long accum134_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard134_0(long x) {
    return x + 8;
}

static long pick134_0_0(long a, long b) { return a > b ? a : b; }
static long pick134_0_1(long a, long b) { return a > b ? a : b; }
static long pick134_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S134_1;

static S134_1 mk134_1(long a) {
    S134_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe134_1(const S134_1 *s) {
    return s->a + s->n0;
}
static long read134_1(const S134_1 *s) {
    return s->a * 4;
}
static void bump134_1(S134_1 *s, long d) {
    s->a = s->a + d;
}
static long classify134_1(int tag, long a, long b) {
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
static long accum134_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard134_1(long x) {
    return x + 5;
}

static long pick134_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S134_2;

static S134_2 mk134_2(long a) {
    S134_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe134_2(const S134_2 *s) {
    return s->a + s->n0;
}
static long read134_2(const S134_2 *s) {
    return s->a * 5;
}
static void bump134_2(S134_2 *s, long d) {
    s->a = s->a + d;
}
static long classify134_2(int tag, long a, long b) {
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
static long accum134_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard134_2(long x) {
    return x + 9;
}

static long pick134_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S134_3;

static S134_3 mk134_3(long a) {
    S134_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe134_3(const S134_3 *s) {
    return s->a + s->n0;
}
static long read134_3(const S134_3 *s) {
    return s->a * 7;
}
static void bump134_3(S134_3 *s, long d) {
    s->a = s->a + d;
}
static long classify134_3(int tag, long a, long b) {
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
static long accum134_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard134_3(long x) {
    return x + 5;
}

static long pick134_3_0(long a, long b) { return a > b ? a : b; }
static long pick134_3_1(long a, long b) { return a > b ? a : b; }
long f134(long x) {
    long acc = x;
    acc += f097(x + 1);
    S134_0 s0 = mk134_0(acc);
    bump134_0(&s0, 5);
    acc += probe134_0(&s0);
    acc += read134_0(&s0);
    acc += classify134_0(1, acc, acc);
    acc += accum134_0(8);
    acc += guard134_0(acc);
    acc += pick134_0_0(acc, acc + 9);
    acc += pick134_0_1(acc, acc + 7);
    acc += pick134_0_2(acc, acc + 8);
    S134_1 s1 = mk134_1(acc);
    bump134_1(&s1, 6);
    acc += probe134_1(&s1);
    acc += read134_1(&s1);
    acc += classify134_1(1, acc, acc);
    acc += accum134_1(6);
    acc += guard134_1(acc);
    acc += pick134_1_0(acc, acc + 5);
    S134_2 s2 = mk134_2(acc);
    bump134_2(&s2, 2);
    acc += probe134_2(&s2);
    acc += read134_2(&s2);
    acc += classify134_2(1, acc, acc);
    acc += accum134_2(3);
    acc += guard134_2(acc);
    acc += pick134_2_0(acc, acc + 3);
    S134_3 s3 = mk134_3(acc);
    bump134_3(&s3, 4);
    acc += probe134_3(&s3);
    acc += read134_3(&s3);
    acc += classify134_3(1, acc, acc);
    acc += accum134_3(7);
    acc += guard134_3(acc);
    acc += pick134_3_0(acc, acc + 1);
    acc += pick134_3_1(acc, acc + 3);
    return clampi(acc);
}
