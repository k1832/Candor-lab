/* GENERATED C mirror of reference module m123. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S123_0;

static S123_0 mk123_0(long a) {
    S123_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe123_0(const S123_0 *s) {
    return s->a + s->n0;
}
static long read123_0(const S123_0 *s) {
    return s->a * 6;
}
static void bump123_0(S123_0 *s, long d) {
    s->a = s->a + d;
}
static long classify123_0(int tag, long a, long b) {
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
static long accum123_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard123_0(long x) {
    return x + 8;
}

static long pick123_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S123_1;

static S123_1 mk123_1(long a) {
    S123_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe123_1(const S123_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read123_1(const S123_1 *s) {
    return s->a * 7;
}
static void bump123_1(S123_1 *s, long d) {
    s->a = s->a + d;
}
static long classify123_1(int tag, long a, long b) {
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
static long accum123_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard123_1(long x) {
    return x + 4;
}

static long pick123_1_0(long a, long b) { return a > b ? a : b; }
static long pick123_1_1(long a, long b) { return a > b ? a : b; }
static long pick123_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S123_2;

static S123_2 mk123_2(long a) {
    S123_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe123_2(const S123_2 *s) {
    return s->a + s->n0;
}
static long read123_2(const S123_2 *s) {
    return s->a * 3;
}
static void bump123_2(S123_2 *s, long d) {
    s->a = s->a + d;
}
static long classify123_2(int tag, long a, long b) {
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
static long accum123_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard123_2(long x) {
    return x + 9;
}

static long pick123_2_0(long a, long b) { return a > b ? a : b; }
static long pick123_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S123_3;

static S123_3 mk123_3(long a) {
    S123_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe123_3(const S123_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read123_3(const S123_3 *s) {
    return s->a * 2;
}
static void bump123_3(S123_3 *s, long d) {
    s->a = s->a + d;
}
static long classify123_3(int tag, long a, long b) {
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
static long accum123_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard123_3(long x) {
    return x + 9;
}

static long pick123_3_0(long a, long b) { return a > b ? a : b; }
static long pick123_3_1(long a, long b) { return a > b ? a : b; }
static long pick123_3_2(long a, long b) { return a > b ? a : b; }
long f123(long x) {
    long acc = x;
    acc += f016(x + 1);
    acc += f029(x + 2);
    acc += f030(x + 3);
    acc += f110(x + 4);
    S123_0 s0 = mk123_0(acc);
    bump123_0(&s0, 8);
    acc += probe123_0(&s0);
    acc += read123_0(&s0);
    acc += classify123_0(1, acc, acc);
    acc += accum123_0(8);
    acc += guard123_0(acc);
    acc += pick123_0_0(acc, acc + 4);
    S123_1 s1 = mk123_1(acc);
    bump123_1(&s1, 9);
    acc += probe123_1(&s1);
    acc += read123_1(&s1);
    acc += classify123_1(1, acc, acc);
    acc += accum123_1(3);
    acc += guard123_1(acc);
    acc += pick123_1_0(acc, acc + 8);
    acc += pick123_1_1(acc, acc + 1);
    acc += pick123_1_2(acc, acc + 9);
    S123_2 s2 = mk123_2(acc);
    bump123_2(&s2, 9);
    acc += probe123_2(&s2);
    acc += read123_2(&s2);
    acc += classify123_2(1, acc, acc);
    acc += accum123_2(4);
    acc += guard123_2(acc);
    acc += pick123_2_0(acc, acc + 4);
    acc += pick123_2_1(acc, acc + 6);
    S123_3 s3 = mk123_3(acc);
    bump123_3(&s3, 3);
    acc += probe123_3(&s3);
    acc += read123_3(&s3);
    acc += classify123_3(1, acc, acc);
    acc += accum123_3(9);
    acc += guard123_3(acc);
    acc += pick123_3_0(acc, acc + 7);
    acc += pick123_3_1(acc, acc + 4);
    acc += pick123_3_2(acc, acc + 1);
    return clampi(acc);
}
