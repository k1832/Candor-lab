/* GENERATED C mirror of reference module m103. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S103_0;

static S103_0 mk103_0(long a) {
    S103_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe103_0(const S103_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read103_0(const S103_0 *s) {
    return s->a * 7;
}
static void bump103_0(S103_0 *s, long d) {
    s->a = s->a + d;
}
static long classify103_0(int tag, long a, long b) {
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
static long accum103_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard103_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S103_1;

static S103_1 mk103_1(long a) {
    S103_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe103_1(const S103_1 *s) {
    return s->a + s->n0;
}
static long read103_1(const S103_1 *s) {
    return s->a * 2;
}
static void bump103_1(S103_1 *s, long d) {
    s->a = s->a + d;
}
static long classify103_1(int tag, long a, long b) {
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
static long accum103_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard103_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S103_2;

static S103_2 mk103_2(long a) {
    S103_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe103_2(const S103_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read103_2(const S103_2 *s) {
    return s->a * 5;
}
static void bump103_2(S103_2 *s, long d) {
    s->a = s->a + d;
}
static long classify103_2(int tag, long a, long b) {
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
static long accum103_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard103_2(long x) {
    return x + 7;
}

long f103(long x) {
    long acc = x;
    acc += f031(x + 1);
    acc += f048(x + 2);
    S103_0 s0 = mk103_0(acc);
    bump103_0(&s0, 4);
    acc += probe103_0(&s0);
    acc += read103_0(&s0);
    acc += classify103_0(1, acc, acc);
    acc += accum103_0(6);
    acc += guard103_0(acc);
    S103_1 s1 = mk103_1(acc);
    bump103_1(&s1, 4);
    acc += probe103_1(&s1);
    acc += read103_1(&s1);
    acc += classify103_1(1, acc, acc);
    acc += accum103_1(9);
    acc += guard103_1(acc);
    S103_2 s2 = mk103_2(acc);
    bump103_2(&s2, 2);
    acc += probe103_2(&s2);
    acc += read103_2(&s2);
    acc += classify103_2(1, acc, acc);
    acc += accum103_2(7);
    acc += guard103_2(acc);
    return clampi(acc);
}
