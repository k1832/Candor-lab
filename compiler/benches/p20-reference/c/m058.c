/* GENERATED C mirror of reference module m058. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S58_0;

static S58_0 mk58_0(long a) {
    S58_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe58_0(const S58_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read58_0(const S58_0 *s) {
    return s->a * 5;
}
static void bump58_0(S58_0 *s, long d) {
    s->a = s->a + d;
}
static long classify58_0(int tag, long a, long b) {
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
static long accum58_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard58_0(long x) {
    return x + 4;
}

static long pick58_0_0(long a, long b) { return a > b ? a : b; }
static long pick58_0_1(long a, long b) { return a > b ? a : b; }
static long pick58_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S58_1;

static S58_1 mk58_1(long a) {
    S58_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe58_1(const S58_1 *s) {
    return s->a + s->n0;
}
static long read58_1(const S58_1 *s) {
    return s->a * 3;
}
static void bump58_1(S58_1 *s, long d) {
    s->a = s->a + d;
}
static long classify58_1(int tag, long a, long b) {
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
static long accum58_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard58_1(long x) {
    return x + 5;
}

static long pick58_1_0(long a, long b) { return a > b ? a : b; }
static long pick58_1_1(long a, long b) { return a > b ? a : b; }
static long pick58_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S58_2;

static S58_2 mk58_2(long a) {
    S58_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe58_2(const S58_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read58_2(const S58_2 *s) {
    return s->a * 5;
}
static void bump58_2(S58_2 *s, long d) {
    s->a = s->a + d;
}
static long classify58_2(int tag, long a, long b) {
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
static long accum58_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard58_2(long x) {
    return x + 8;
}

static long pick58_2_0(long a, long b) { return a > b ? a : b; }
static long pick58_2_1(long a, long b) { return a > b ? a : b; }
static long pick58_2_2(long a, long b) { return a > b ? a : b; }
long f058(long x) {
    long acc = x;
    acc += f018(x + 1);
    acc += f041(x + 2);
    S58_0 s0 = mk58_0(acc);
    bump58_0(&s0, 8);
    acc += probe58_0(&s0);
    acc += read58_0(&s0);
    acc += classify58_0(1, acc, acc);
    acc += accum58_0(3);
    acc += guard58_0(acc);
    acc += pick58_0_0(acc, acc + 8);
    acc += pick58_0_1(acc, acc + 5);
    acc += pick58_0_2(acc, acc + 1);
    S58_1 s1 = mk58_1(acc);
    bump58_1(&s1, 7);
    acc += probe58_1(&s1);
    acc += read58_1(&s1);
    acc += classify58_1(1, acc, acc);
    acc += accum58_1(3);
    acc += guard58_1(acc);
    acc += pick58_1_0(acc, acc + 6);
    acc += pick58_1_1(acc, acc + 7);
    acc += pick58_1_2(acc, acc + 4);
    S58_2 s2 = mk58_2(acc);
    bump58_2(&s2, 9);
    acc += probe58_2(&s2);
    acc += read58_2(&s2);
    acc += classify58_2(1, acc, acc);
    acc += accum58_2(5);
    acc += guard58_2(acc);
    acc += pick58_2_0(acc, acc + 1);
    acc += pick58_2_1(acc, acc + 1);
    acc += pick58_2_2(acc, acc + 1);
    return clampi(acc);
}
