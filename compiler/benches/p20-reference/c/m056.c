/* GENERATED C mirror of reference module m056. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S56_0;

static S56_0 mk56_0(long a) {
    S56_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe56_0(const S56_0 *s) {
    return s->a + s->n0;
}
static long read56_0(const S56_0 *s) {
    return s->a * 3;
}
static void bump56_0(S56_0 *s, long d) {
    s->a = s->a + d;
}
static long classify56_0(int tag, long a, long b) {
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
static long accum56_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard56_0(long x) {
    return x + 9;
}

static long pick56_0_0(long a, long b) { return a > b ? a : b; }
static long pick56_0_1(long a, long b) { return a > b ? a : b; }
static long pick56_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S56_1;

static S56_1 mk56_1(long a) {
    S56_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe56_1(const S56_1 *s) {
    return s->a + s->n0;
}
static long read56_1(const S56_1 *s) {
    return s->a * 4;
}
static void bump56_1(S56_1 *s, long d) {
    s->a = s->a + d;
}
static long classify56_1(int tag, long a, long b) {
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
static long accum56_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard56_1(long x) {
    return x + 4;
}

static long pick56_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S56_2;

static S56_2 mk56_2(long a) {
    S56_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe56_2(const S56_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read56_2(const S56_2 *s) {
    return s->a * 4;
}
static void bump56_2(S56_2 *s, long d) {
    s->a = s->a + d;
}
static long classify56_2(int tag, long a, long b) {
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
static long accum56_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard56_2(long x) {
    return x + 3;
}

static long pick56_2_0(long a, long b) { return a > b ? a : b; }
static long pick56_2_1(long a, long b) { return a > b ? a : b; }
static long pick56_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S56_3;

static S56_3 mk56_3(long a) {
    S56_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe56_3(const S56_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read56_3(const S56_3 *s) {
    return s->a * 5;
}
static void bump56_3(S56_3 *s, long d) {
    s->a = s->a + d;
}
static long classify56_3(int tag, long a, long b) {
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
static long accum56_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard56_3(long x) {
    return x + 3;
}

static long pick56_3_0(long a, long b) { return a > b ? a : b; }
static long pick56_3_1(long a, long b) { return a > b ? a : b; }
static long pick56_3_2(long a, long b) { return a > b ? a : b; }
long f056(long x) {
    long acc = x;
    acc += f006(x + 1);
    acc += f013(x + 2);
    acc += f020(x + 3);
    acc += f044(x + 4);
    S56_0 s0 = mk56_0(acc);
    bump56_0(&s0, 7);
    acc += probe56_0(&s0);
    acc += read56_0(&s0);
    acc += classify56_0(1, acc, acc);
    acc += accum56_0(4);
    acc += guard56_0(acc);
    acc += pick56_0_0(acc, acc + 9);
    acc += pick56_0_1(acc, acc + 4);
    acc += pick56_0_2(acc, acc + 4);
    S56_1 s1 = mk56_1(acc);
    bump56_1(&s1, 8);
    acc += probe56_1(&s1);
    acc += read56_1(&s1);
    acc += classify56_1(1, acc, acc);
    acc += accum56_1(5);
    acc += guard56_1(acc);
    acc += pick56_1_0(acc, acc + 8);
    S56_2 s2 = mk56_2(acc);
    bump56_2(&s2, 5);
    acc += probe56_2(&s2);
    acc += read56_2(&s2);
    acc += classify56_2(1, acc, acc);
    acc += accum56_2(4);
    acc += guard56_2(acc);
    acc += pick56_2_0(acc, acc + 6);
    acc += pick56_2_1(acc, acc + 8);
    acc += pick56_2_2(acc, acc + 4);
    S56_3 s3 = mk56_3(acc);
    bump56_3(&s3, 4);
    acc += probe56_3(&s3);
    acc += read56_3(&s3);
    acc += classify56_3(1, acc, acc);
    acc += accum56_3(5);
    acc += guard56_3(acc);
    acc += pick56_3_0(acc, acc + 6);
    acc += pick56_3_1(acc, acc + 6);
    acc += pick56_3_2(acc, acc + 3);
    return clampi(acc);
}
