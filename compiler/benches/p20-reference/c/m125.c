/* GENERATED C mirror of reference module m125. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S125_0;

static S125_0 mk125_0(long a) {
    S125_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe125_0(const S125_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read125_0(const S125_0 *s) {
    return s->a * 7;
}
static void bump125_0(S125_0 *s, long d) {
    s->a = s->a + d;
}
static long classify125_0(int tag, long a, long b) {
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
static long accum125_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard125_0(long x) {
    return x + 7;
}

static long pick125_0_0(long a, long b) { return a > b ? a : b; }
static long pick125_0_1(long a, long b) { return a > b ? a : b; }
static long pick125_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S125_1;

static S125_1 mk125_1(long a) {
    S125_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe125_1(const S125_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read125_1(const S125_1 *s) {
    return s->a * 6;
}
static void bump125_1(S125_1 *s, long d) {
    s->a = s->a + d;
}
static long classify125_1(int tag, long a, long b) {
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
static long accum125_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard125_1(long x) {
    return x + 4;
}

static long pick125_1_0(long a, long b) { return a > b ? a : b; }
static long pick125_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S125_2;

static S125_2 mk125_2(long a) {
    S125_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe125_2(const S125_2 *s) {
    return s->a + s->n0;
}
static long read125_2(const S125_2 *s) {
    return s->a * 6;
}
static void bump125_2(S125_2 *s, long d) {
    s->a = s->a + d;
}
static long classify125_2(int tag, long a, long b) {
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
static long accum125_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard125_2(long x) {
    return x + 6;
}

static long pick125_2_0(long a, long b) { return a > b ? a : b; }
static long pick125_2_1(long a, long b) { return a > b ? a : b; }
static long pick125_2_2(long a, long b) { return a > b ? a : b; }
long f125(long x) {
    long acc = x;
    acc += f060(x + 1);
    S125_0 s0 = mk125_0(acc);
    bump125_0(&s0, 5);
    acc += probe125_0(&s0);
    acc += read125_0(&s0);
    acc += classify125_0(1, acc, acc);
    acc += accum125_0(5);
    acc += guard125_0(acc);
    acc += pick125_0_0(acc, acc + 6);
    acc += pick125_0_1(acc, acc + 7);
    acc += pick125_0_2(acc, acc + 2);
    S125_1 s1 = mk125_1(acc);
    bump125_1(&s1, 4);
    acc += probe125_1(&s1);
    acc += read125_1(&s1);
    acc += classify125_1(1, acc, acc);
    acc += accum125_1(8);
    acc += guard125_1(acc);
    acc += pick125_1_0(acc, acc + 4);
    acc += pick125_1_1(acc, acc + 3);
    S125_2 s2 = mk125_2(acc);
    bump125_2(&s2, 8);
    acc += probe125_2(&s2);
    acc += read125_2(&s2);
    acc += classify125_2(1, acc, acc);
    acc += accum125_2(3);
    acc += guard125_2(acc);
    acc += pick125_2_0(acc, acc + 7);
    acc += pick125_2_1(acc, acc + 4);
    acc += pick125_2_2(acc, acc + 3);
    return clampi(acc);
}
