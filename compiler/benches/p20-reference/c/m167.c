/* GENERATED C mirror of reference module m167. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S167_0;

static S167_0 mk167_0(long a) {
    S167_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe167_0(const S167_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read167_0(const S167_0 *s) {
    return s->a * 5;
}
static void bump167_0(S167_0 *s, long d) {
    s->a = s->a + d;
}
static long classify167_0(int tag, long a, long b) {
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
static long accum167_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard167_0(long x) {
    return x + 8;
}

static long pick167_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S167_1;

static S167_1 mk167_1(long a) {
    S167_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe167_1(const S167_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read167_1(const S167_1 *s) {
    return s->a * 6;
}
static void bump167_1(S167_1 *s, long d) {
    s->a = s->a + d;
}
static long classify167_1(int tag, long a, long b) {
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
static long accum167_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard167_1(long x) {
    return x + 1;
}

static long pick167_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S167_2;

static S167_2 mk167_2(long a) {
    S167_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe167_2(const S167_2 *s) {
    return s->a + s->n0;
}
static long read167_2(const S167_2 *s) {
    return s->a * 3;
}
static void bump167_2(S167_2 *s, long d) {
    s->a = s->a + d;
}
static long classify167_2(int tag, long a, long b) {
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
static long accum167_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard167_2(long x) {
    return x + 6;
}

static long pick167_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S167_3;

static S167_3 mk167_3(long a) {
    S167_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe167_3(const S167_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read167_3(const S167_3 *s) {
    return s->a * 5;
}
static void bump167_3(S167_3 *s, long d) {
    s->a = s->a + d;
}
static long classify167_3(int tag, long a, long b) {
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
static long accum167_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard167_3(long x) {
    return x + 9;
}

static long pick167_3_0(long a, long b) { return a > b ? a : b; }
static long pick167_3_1(long a, long b) { return a > b ? a : b; }
long f167(long x) {
    long acc = x;
    acc += f027(x + 1);
    S167_0 s0 = mk167_0(acc);
    bump167_0(&s0, 3);
    acc += probe167_0(&s0);
    acc += read167_0(&s0);
    acc += classify167_0(1, acc, acc);
    acc += accum167_0(7);
    acc += guard167_0(acc);
    acc += pick167_0_0(acc, acc + 4);
    S167_1 s1 = mk167_1(acc);
    bump167_1(&s1, 9);
    acc += probe167_1(&s1);
    acc += read167_1(&s1);
    acc += classify167_1(1, acc, acc);
    acc += accum167_1(6);
    acc += guard167_1(acc);
    acc += pick167_1_0(acc, acc + 6);
    S167_2 s2 = mk167_2(acc);
    bump167_2(&s2, 6);
    acc += probe167_2(&s2);
    acc += read167_2(&s2);
    acc += classify167_2(1, acc, acc);
    acc += accum167_2(8);
    acc += guard167_2(acc);
    acc += pick167_2_0(acc, acc + 2);
    S167_3 s3 = mk167_3(acc);
    bump167_3(&s3, 5);
    acc += probe167_3(&s3);
    acc += read167_3(&s3);
    acc += classify167_3(1, acc, acc);
    acc += accum167_3(4);
    acc += guard167_3(acc);
    acc += pick167_3_0(acc, acc + 8);
    acc += pick167_3_1(acc, acc + 7);
    return clampi(acc);
}
