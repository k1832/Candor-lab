/* GENERATED C mirror of reference module m019. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S19_0;

static S19_0 mk19_0(long a) {
    S19_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe19_0(const S19_0 *s) {
    return s->a + s->n0;
}
static long read19_0(const S19_0 *s) {
    return s->a * 7;
}
static void bump19_0(S19_0 *s, long d) {
    s->a = s->a + d;
}
static long classify19_0(int tag, long a, long b) {
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
static long accum19_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard19_0(long x) {
    return x + 7;
}

static long pick19_0_0(long a, long b) { return a > b ? a : b; }
static long pick19_0_1(long a, long b) { return a > b ? a : b; }
static long pick19_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S19_1;

static S19_1 mk19_1(long a) {
    S19_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe19_1(const S19_1 *s) {
    return s->a + s->n0;
}
static long read19_1(const S19_1 *s) {
    return s->a * 4;
}
static void bump19_1(S19_1 *s, long d) {
    s->a = s->a + d;
}
static long classify19_1(int tag, long a, long b) {
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
static long accum19_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard19_1(long x) {
    return x + 6;
}

static long pick19_1_0(long a, long b) { return a > b ? a : b; }
static long pick19_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S19_2;

static S19_2 mk19_2(long a) {
    S19_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe19_2(const S19_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read19_2(const S19_2 *s) {
    return s->a * 7;
}
static void bump19_2(S19_2 *s, long d) {
    s->a = s->a + d;
}
static long classify19_2(int tag, long a, long b) {
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
static long accum19_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard19_2(long x) {
    return x + 9;
}

static long pick19_2_0(long a, long b) { return a > b ? a : b; }
long f019(long x) {
    long acc = x;
    acc += f000(x + 1);
    S19_0 s0 = mk19_0(acc);
    bump19_0(&s0, 2);
    acc += probe19_0(&s0);
    acc += read19_0(&s0);
    acc += classify19_0(1, acc, acc);
    acc += accum19_0(7);
    acc += guard19_0(acc);
    acc += pick19_0_0(acc, acc + 9);
    acc += pick19_0_1(acc, acc + 8);
    acc += pick19_0_2(acc, acc + 9);
    S19_1 s1 = mk19_1(acc);
    bump19_1(&s1, 8);
    acc += probe19_1(&s1);
    acc += read19_1(&s1);
    acc += classify19_1(1, acc, acc);
    acc += accum19_1(4);
    acc += guard19_1(acc);
    acc += pick19_1_0(acc, acc + 7);
    acc += pick19_1_1(acc, acc + 1);
    S19_2 s2 = mk19_2(acc);
    bump19_2(&s2, 8);
    acc += probe19_2(&s2);
    acc += read19_2(&s2);
    acc += classify19_2(1, acc, acc);
    acc += accum19_2(7);
    acc += guard19_2(acc);
    acc += pick19_2_0(acc, acc + 8);
    return clampi(acc);
}
