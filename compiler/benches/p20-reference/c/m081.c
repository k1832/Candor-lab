/* GENERATED C mirror of reference module m081. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S81_0;

static S81_0 mk81_0(long a) {
    S81_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe81_0(const S81_0 *s) {
    return s->a + s->n0;
}
static long read81_0(const S81_0 *s) {
    return s->a * 5;
}
static void bump81_0(S81_0 *s, long d) {
    s->a = s->a + d;
}
static long classify81_0(int tag, long a, long b) {
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
static long accum81_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard81_0(long x) {
    return x + 7;
}

static long pick81_0_0(long a, long b) { return a > b ? a : b; }
static long pick81_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S81_1;

static S81_1 mk81_1(long a) {
    S81_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe81_1(const S81_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read81_1(const S81_1 *s) {
    return s->a * 3;
}
static void bump81_1(S81_1 *s, long d) {
    s->a = s->a + d;
}
static long classify81_1(int tag, long a, long b) {
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
static long accum81_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard81_1(long x) {
    return x + 8;
}

static long pick81_1_0(long a, long b) { return a > b ? a : b; }
static long pick81_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S81_2;

static S81_2 mk81_2(long a) {
    S81_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe81_2(const S81_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read81_2(const S81_2 *s) {
    return s->a * 3;
}
static void bump81_2(S81_2 *s, long d) {
    s->a = s->a + d;
}
static long classify81_2(int tag, long a, long b) {
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
static long accum81_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard81_2(long x) {
    return x + 6;
}

static long pick81_2_0(long a, long b) { return a > b ? a : b; }
static long pick81_2_1(long a, long b) { return a > b ? a : b; }
static long pick81_2_2(long a, long b) { return a > b ? a : b; }
long f081(long x) {
    long acc = x;
    acc += f040(x + 1);
    acc += f049(x + 2);
    S81_0 s0 = mk81_0(acc);
    bump81_0(&s0, 7);
    acc += probe81_0(&s0);
    acc += read81_0(&s0);
    acc += classify81_0(1, acc, acc);
    acc += accum81_0(8);
    acc += guard81_0(acc);
    acc += pick81_0_0(acc, acc + 3);
    acc += pick81_0_1(acc, acc + 4);
    S81_1 s1 = mk81_1(acc);
    bump81_1(&s1, 6);
    acc += probe81_1(&s1);
    acc += read81_1(&s1);
    acc += classify81_1(1, acc, acc);
    acc += accum81_1(7);
    acc += guard81_1(acc);
    acc += pick81_1_0(acc, acc + 3);
    acc += pick81_1_1(acc, acc + 9);
    S81_2 s2 = mk81_2(acc);
    bump81_2(&s2, 7);
    acc += probe81_2(&s2);
    acc += read81_2(&s2);
    acc += classify81_2(1, acc, acc);
    acc += accum81_2(4);
    acc += guard81_2(acc);
    acc += pick81_2_0(acc, acc + 6);
    acc += pick81_2_1(acc, acc + 4);
    acc += pick81_2_2(acc, acc + 9);
    return clampi(acc);
}
