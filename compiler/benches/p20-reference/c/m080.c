/* GENERATED C mirror of reference module m080. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S80_0;

static S80_0 mk80_0(long a) {
    S80_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe80_0(const S80_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read80_0(const S80_0 *s) {
    return s->a * 6;
}
static void bump80_0(S80_0 *s, long d) {
    s->a = s->a + d;
}
static long classify80_0(int tag, long a, long b) {
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
static long accum80_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard80_0(long x) {
    return x + 6;
}

static long pick80_0_0(long a, long b) { return a > b ? a : b; }
static long pick80_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S80_1;

static S80_1 mk80_1(long a) {
    S80_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe80_1(const S80_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read80_1(const S80_1 *s) {
    return s->a * 6;
}
static void bump80_1(S80_1 *s, long d) {
    s->a = s->a + d;
}
static long classify80_1(int tag, long a, long b) {
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
static long accum80_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard80_1(long x) {
    return x + 5;
}

static long pick80_1_0(long a, long b) { return a > b ? a : b; }
static long pick80_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S80_2;

static S80_2 mk80_2(long a) {
    S80_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe80_2(const S80_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read80_2(const S80_2 *s) {
    return s->a * 7;
}
static void bump80_2(S80_2 *s, long d) {
    s->a = s->a + d;
}
static long classify80_2(int tag, long a, long b) {
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
static long accum80_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard80_2(long x) {
    return x + 3;
}

static long pick80_2_0(long a, long b) { return a > b ? a : b; }
long f080(long x) {
    long acc = x;
    acc += f064(x + 1);
    acc += f069(x + 2);
    S80_0 s0 = mk80_0(acc);
    bump80_0(&s0, 2);
    acc += probe80_0(&s0);
    acc += read80_0(&s0);
    acc += classify80_0(1, acc, acc);
    acc += accum80_0(7);
    acc += guard80_0(acc);
    acc += pick80_0_0(acc, acc + 5);
    acc += pick80_0_1(acc, acc + 7);
    S80_1 s1 = mk80_1(acc);
    bump80_1(&s1, 4);
    acc += probe80_1(&s1);
    acc += read80_1(&s1);
    acc += classify80_1(1, acc, acc);
    acc += accum80_1(4);
    acc += guard80_1(acc);
    acc += pick80_1_0(acc, acc + 9);
    acc += pick80_1_1(acc, acc + 1);
    S80_2 s2 = mk80_2(acc);
    bump80_2(&s2, 8);
    acc += probe80_2(&s2);
    acc += read80_2(&s2);
    acc += classify80_2(1, acc, acc);
    acc += accum80_2(7);
    acc += guard80_2(acc);
    acc += pick80_2_0(acc, acc + 2);
    return clampi(acc);
}
