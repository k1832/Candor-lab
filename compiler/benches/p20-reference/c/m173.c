/* GENERATED C mirror of reference module m173. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S173_0;

static S173_0 mk173_0(long a) {
    S173_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe173_0(const S173_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read173_0(const S173_0 *s) {
    return s->a * 6;
}
static void bump173_0(S173_0 *s, long d) {
    s->a = s->a + d;
}
static long classify173_0(int tag, long a, long b) {
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
static long accum173_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard173_0(long x) {
    return x + 9;
}

static long pick173_0_0(long a, long b) { return a > b ? a : b; }
static long pick173_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S173_1;

static S173_1 mk173_1(long a) {
    S173_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe173_1(const S173_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read173_1(const S173_1 *s) {
    return s->a * 2;
}
static void bump173_1(S173_1 *s, long d) {
    s->a = s->a + d;
}
static long classify173_1(int tag, long a, long b) {
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
static long accum173_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard173_1(long x) {
    return x + 8;
}

static long pick173_1_0(long a, long b) { return a > b ? a : b; }
static long pick173_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S173_2;

static S173_2 mk173_2(long a) {
    S173_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe173_2(const S173_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read173_2(const S173_2 *s) {
    return s->a * 7;
}
static void bump173_2(S173_2 *s, long d) {
    s->a = s->a + d;
}
static long classify173_2(int tag, long a, long b) {
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
static long accum173_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard173_2(long x) {
    return x + 6;
}

static long pick173_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S173_3;

static S173_3 mk173_3(long a) {
    S173_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe173_3(const S173_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read173_3(const S173_3 *s) {
    return s->a * 3;
}
static void bump173_3(S173_3 *s, long d) {
    s->a = s->a + d;
}
static long classify173_3(int tag, long a, long b) {
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
static long accum173_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard173_3(long x) {
    return x + 4;
}

static long pick173_3_0(long a, long b) { return a > b ? a : b; }
long f173(long x) {
    long acc = x;
    acc += f078(x + 1);
    acc += f090(x + 2);
    S173_0 s0 = mk173_0(acc);
    bump173_0(&s0, 7);
    acc += probe173_0(&s0);
    acc += read173_0(&s0);
    acc += classify173_0(1, acc, acc);
    acc += accum173_0(9);
    acc += guard173_0(acc);
    acc += pick173_0_0(acc, acc + 3);
    acc += pick173_0_1(acc, acc + 7);
    S173_1 s1 = mk173_1(acc);
    bump173_1(&s1, 1);
    acc += probe173_1(&s1);
    acc += read173_1(&s1);
    acc += classify173_1(1, acc, acc);
    acc += accum173_1(7);
    acc += guard173_1(acc);
    acc += pick173_1_0(acc, acc + 8);
    acc += pick173_1_1(acc, acc + 9);
    S173_2 s2 = mk173_2(acc);
    bump173_2(&s2, 5);
    acc += probe173_2(&s2);
    acc += read173_2(&s2);
    acc += classify173_2(1, acc, acc);
    acc += accum173_2(4);
    acc += guard173_2(acc);
    acc += pick173_2_0(acc, acc + 1);
    S173_3 s3 = mk173_3(acc);
    bump173_3(&s3, 6);
    acc += probe173_3(&s3);
    acc += read173_3(&s3);
    acc += classify173_3(1, acc, acc);
    acc += accum173_3(8);
    acc += guard173_3(acc);
    acc += pick173_3_0(acc, acc + 8);
    return clampi(acc);
}
