/* GENERATED C mirror of reference module m036. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S36_0;

static S36_0 mk36_0(long a) {
    S36_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe36_0(const S36_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read36_0(const S36_0 *s) {
    return s->a * 7;
}
static void bump36_0(S36_0 *s, long d) {
    s->a = s->a + d;
}
static long classify36_0(int tag, long a, long b) {
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
static long accum36_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard36_0(long x) {
    return x + 8;
}

static long pick36_0_0(long a, long b) { return a > b ? a : b; }
static long pick36_0_1(long a, long b) { return a > b ? a : b; }
static long pick36_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S36_1;

static S36_1 mk36_1(long a) {
    S36_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe36_1(const S36_1 *s) {
    return s->a + s->n0;
}
static long read36_1(const S36_1 *s) {
    return s->a * 4;
}
static void bump36_1(S36_1 *s, long d) {
    s->a = s->a + d;
}
static long classify36_1(int tag, long a, long b) {
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
static long accum36_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard36_1(long x) {
    return x + 4;
}

static long pick36_1_0(long a, long b) { return a > b ? a : b; }
static long pick36_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S36_2;

static S36_2 mk36_2(long a) {
    S36_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe36_2(const S36_2 *s) {
    return s->a + s->n0;
}
static long read36_2(const S36_2 *s) {
    return s->a * 5;
}
static void bump36_2(S36_2 *s, long d) {
    s->a = s->a + d;
}
static long classify36_2(int tag, long a, long b) {
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
static long accum36_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard36_2(long x) {
    return x + 2;
}

static long pick36_2_0(long a, long b) { return a > b ? a : b; }
static long pick36_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S36_3;

static S36_3 mk36_3(long a) {
    S36_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe36_3(const S36_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read36_3(const S36_3 *s) {
    return s->a * 2;
}
static void bump36_3(S36_3 *s, long d) {
    s->a = s->a + d;
}
static long classify36_3(int tag, long a, long b) {
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
static long accum36_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard36_3(long x) {
    return x + 7;
}

static long pick36_3_0(long a, long b) { return a > b ? a : b; }
static long pick36_3_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S36_4;

static S36_4 mk36_4(long a) {
    S36_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe36_4(const S36_4 *s) {
    return s->a + s->n0;
}
static long read36_4(const S36_4 *s) {
    return s->a * 5;
}
static void bump36_4(S36_4 *s, long d) {
    s->a = s->a + d;
}
static long classify36_4(int tag, long a, long b) {
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
static long accum36_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard36_4(long x) {
    return x + 9;
}

static long pick36_4_0(long a, long b) { return a > b ? a : b; }
static long pick36_4_1(long a, long b) { return a > b ? a : b; }
static long pick36_4_2(long a, long b) { return a > b ? a : b; }
long f036(long x) {
    long acc = x;
    acc += f005(x + 1);
    acc += f015(x + 2);
    acc += f022(x + 3);
    S36_0 s0 = mk36_0(acc);
    bump36_0(&s0, 1);
    acc += probe36_0(&s0);
    acc += read36_0(&s0);
    acc += classify36_0(1, acc, acc);
    acc += accum36_0(7);
    acc += guard36_0(acc);
    acc += pick36_0_0(acc, acc + 4);
    acc += pick36_0_1(acc, acc + 9);
    acc += pick36_0_2(acc, acc + 3);
    S36_1 s1 = mk36_1(acc);
    bump36_1(&s1, 3);
    acc += probe36_1(&s1);
    acc += read36_1(&s1);
    acc += classify36_1(1, acc, acc);
    acc += accum36_1(8);
    acc += guard36_1(acc);
    acc += pick36_1_0(acc, acc + 2);
    acc += pick36_1_1(acc, acc + 1);
    S36_2 s2 = mk36_2(acc);
    bump36_2(&s2, 5);
    acc += probe36_2(&s2);
    acc += read36_2(&s2);
    acc += classify36_2(1, acc, acc);
    acc += accum36_2(4);
    acc += guard36_2(acc);
    acc += pick36_2_0(acc, acc + 2);
    acc += pick36_2_1(acc, acc + 6);
    S36_3 s3 = mk36_3(acc);
    bump36_3(&s3, 5);
    acc += probe36_3(&s3);
    acc += read36_3(&s3);
    acc += classify36_3(1, acc, acc);
    acc += accum36_3(6);
    acc += guard36_3(acc);
    acc += pick36_3_0(acc, acc + 9);
    acc += pick36_3_1(acc, acc + 4);
    S36_4 s4 = mk36_4(acc);
    bump36_4(&s4, 7);
    acc += probe36_4(&s4);
    acc += read36_4(&s4);
    acc += classify36_4(1, acc, acc);
    acc += accum36_4(5);
    acc += guard36_4(acc);
    acc += pick36_4_0(acc, acc + 7);
    acc += pick36_4_1(acc, acc + 8);
    acc += pick36_4_2(acc, acc + 8);
    return clampi(acc);
}
