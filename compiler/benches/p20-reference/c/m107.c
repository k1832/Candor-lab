/* GENERATED C mirror of reference module m107. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S107_0;

static S107_0 mk107_0(long a) {
    S107_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe107_0(const S107_0 *s) {
    return s->a + s->n0;
}
static long read107_0(const S107_0 *s) {
    return s->a * 2;
}
static void bump107_0(S107_0 *s, long d) {
    s->a = s->a + d;
}
static long classify107_0(int tag, long a, long b) {
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
static long accum107_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard107_0(long x) {
    return x + 8;
}

static long pick107_0_0(long a, long b) { return a > b ? a : b; }
static long pick107_0_1(long a, long b) { return a > b ? a : b; }
static long pick107_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S107_1;

static S107_1 mk107_1(long a) {
    S107_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe107_1(const S107_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read107_1(const S107_1 *s) {
    return s->a * 2;
}
static void bump107_1(S107_1 *s, long d) {
    s->a = s->a + d;
}
static long classify107_1(int tag, long a, long b) {
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
static long accum107_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard107_1(long x) {
    return x + 8;
}

static long pick107_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S107_2;

static S107_2 mk107_2(long a) {
    S107_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe107_2(const S107_2 *s) {
    return s->a + s->n0;
}
static long read107_2(const S107_2 *s) {
    return s->a * 4;
}
static void bump107_2(S107_2 *s, long d) {
    s->a = s->a + d;
}
static long classify107_2(int tag, long a, long b) {
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
static long accum107_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard107_2(long x) {
    return x + 4;
}

static long pick107_2_0(long a, long b) { return a > b ? a : b; }
long f107(long x) {
    long acc = x;
    acc += f002(x + 1);
    acc += f019(x + 2);
    acc += f059(x + 3);
    acc += f069(x + 4);
    S107_0 s0 = mk107_0(acc);
    bump107_0(&s0, 7);
    acc += probe107_0(&s0);
    acc += read107_0(&s0);
    acc += classify107_0(1, acc, acc);
    acc += accum107_0(5);
    acc += guard107_0(acc);
    acc += pick107_0_0(acc, acc + 7);
    acc += pick107_0_1(acc, acc + 6);
    acc += pick107_0_2(acc, acc + 7);
    S107_1 s1 = mk107_1(acc);
    bump107_1(&s1, 1);
    acc += probe107_1(&s1);
    acc += read107_1(&s1);
    acc += classify107_1(1, acc, acc);
    acc += accum107_1(3);
    acc += guard107_1(acc);
    acc += pick107_1_0(acc, acc + 9);
    S107_2 s2 = mk107_2(acc);
    bump107_2(&s2, 9);
    acc += probe107_2(&s2);
    acc += read107_2(&s2);
    acc += classify107_2(1, acc, acc);
    acc += accum107_2(6);
    acc += guard107_2(acc);
    acc += pick107_2_0(acc, acc + 7);
    return clampi(acc);
}
