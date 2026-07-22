/* GENERATED C mirror of reference module m064. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S64_0;

static S64_0 mk64_0(long a) {
    S64_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe64_0(const S64_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read64_0(const S64_0 *s) {
    return s->a * 4;
}
static void bump64_0(S64_0 *s, long d) {
    s->a = s->a + d;
}
static long classify64_0(int tag, long a, long b) {
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
static long accum64_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard64_0(long x) {
    return x + 2;
}

static long pick64_0_0(long a, long b) { return a > b ? a : b; }
static long pick64_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S64_1;

static S64_1 mk64_1(long a) {
    S64_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe64_1(const S64_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read64_1(const S64_1 *s) {
    return s->a * 5;
}
static void bump64_1(S64_1 *s, long d) {
    s->a = s->a + d;
}
static long classify64_1(int tag, long a, long b) {
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
static long accum64_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard64_1(long x) {
    return x + 1;
}

static long pick64_1_0(long a, long b) { return a > b ? a : b; }
static long pick64_1_1(long a, long b) { return a > b ? a : b; }
static long pick64_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S64_2;

static S64_2 mk64_2(long a) {
    S64_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe64_2(const S64_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read64_2(const S64_2 *s) {
    return s->a * 5;
}
static void bump64_2(S64_2 *s, long d) {
    s->a = s->a + d;
}
static long classify64_2(int tag, long a, long b) {
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
static long accum64_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard64_2(long x) {
    return x + 4;
}

static long pick64_2_0(long a, long b) { return a > b ? a : b; }
static long pick64_2_1(long a, long b) { return a > b ? a : b; }
static long pick64_2_2(long a, long b) { return a > b ? a : b; }
long f064(long x) {
    long acc = x;
    acc += f008(x + 1);
    acc += f010(x + 2);
    acc += f019(x + 3);
    acc += f023(x + 4);
    S64_0 s0 = mk64_0(acc);
    bump64_0(&s0, 3);
    acc += probe64_0(&s0);
    acc += read64_0(&s0);
    acc += classify64_0(1, acc, acc);
    acc += accum64_0(5);
    acc += guard64_0(acc);
    acc += pick64_0_0(acc, acc + 9);
    acc += pick64_0_1(acc, acc + 8);
    S64_1 s1 = mk64_1(acc);
    bump64_1(&s1, 6);
    acc += probe64_1(&s1);
    acc += read64_1(&s1);
    acc += classify64_1(1, acc, acc);
    acc += accum64_1(6);
    acc += guard64_1(acc);
    acc += pick64_1_0(acc, acc + 1);
    acc += pick64_1_1(acc, acc + 3);
    acc += pick64_1_2(acc, acc + 9);
    S64_2 s2 = mk64_2(acc);
    bump64_2(&s2, 6);
    acc += probe64_2(&s2);
    acc += read64_2(&s2);
    acc += classify64_2(1, acc, acc);
    acc += accum64_2(8);
    acc += guard64_2(acc);
    acc += pick64_2_0(acc, acc + 4);
    acc += pick64_2_1(acc, acc + 4);
    acc += pick64_2_2(acc, acc + 7);
    return clampi(acc);
}
