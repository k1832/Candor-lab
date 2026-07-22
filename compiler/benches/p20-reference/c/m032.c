/* GENERATED C mirror of reference module m032. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S32_0;

static S32_0 mk32_0(long a) {
    S32_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe32_0(const S32_0 *s) {
    return s->a + s->n0;
}
static long read32_0(const S32_0 *s) {
    return s->a * 5;
}
static void bump32_0(S32_0 *s, long d) {
    s->a = s->a + d;
}
static long classify32_0(int tag, long a, long b) {
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
static long accum32_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard32_0(long x) {
    return x + 2;
}

static long pick32_0_0(long a, long b) { return a > b ? a : b; }
static long pick32_0_1(long a, long b) { return a > b ? a : b; }
static long pick32_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S32_1;

static S32_1 mk32_1(long a) {
    S32_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe32_1(const S32_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read32_1(const S32_1 *s) {
    return s->a * 6;
}
static void bump32_1(S32_1 *s, long d) {
    s->a = s->a + d;
}
static long classify32_1(int tag, long a, long b) {
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
static long accum32_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard32_1(long x) {
    return x + 2;
}

static long pick32_1_0(long a, long b) { return a > b ? a : b; }
static long pick32_1_1(long a, long b) { return a > b ? a : b; }
static long pick32_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S32_2;

static S32_2 mk32_2(long a) {
    S32_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe32_2(const S32_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read32_2(const S32_2 *s) {
    return s->a * 5;
}
static void bump32_2(S32_2 *s, long d) {
    s->a = s->a + d;
}
static long classify32_2(int tag, long a, long b) {
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
static long accum32_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard32_2(long x) {
    return x + 6;
}

static long pick32_2_0(long a, long b) { return a > b ? a : b; }
static long pick32_2_1(long a, long b) { return a > b ? a : b; }
long f032(long x) {
    long acc = x;
    acc += f000(x + 1);
    acc += f015(x + 2);
    S32_0 s0 = mk32_0(acc);
    bump32_0(&s0, 3);
    acc += probe32_0(&s0);
    acc += read32_0(&s0);
    acc += classify32_0(1, acc, acc);
    acc += accum32_0(7);
    acc += guard32_0(acc);
    acc += pick32_0_0(acc, acc + 6);
    acc += pick32_0_1(acc, acc + 2);
    acc += pick32_0_2(acc, acc + 6);
    S32_1 s1 = mk32_1(acc);
    bump32_1(&s1, 2);
    acc += probe32_1(&s1);
    acc += read32_1(&s1);
    acc += classify32_1(1, acc, acc);
    acc += accum32_1(7);
    acc += guard32_1(acc);
    acc += pick32_1_0(acc, acc + 9);
    acc += pick32_1_1(acc, acc + 6);
    acc += pick32_1_2(acc, acc + 8);
    S32_2 s2 = mk32_2(acc);
    bump32_2(&s2, 7);
    acc += probe32_2(&s2);
    acc += read32_2(&s2);
    acc += classify32_2(1, acc, acc);
    acc += accum32_2(6);
    acc += guard32_2(acc);
    acc += pick32_2_0(acc, acc + 1);
    acc += pick32_2_1(acc, acc + 7);
    return clampi(acc);
}
