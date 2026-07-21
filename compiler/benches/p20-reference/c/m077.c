/* GENERATED C mirror of reference module m077. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S77_0;

static S77_0 mk77_0(long a) {
    S77_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe77_0(const S77_0 *s) {
    return s->a + s->n0;
}
static long read77_0(const S77_0 *s) {
    return s->a * 6;
}
static void bump77_0(S77_0 *s, long d) {
    s->a = s->a + d;
}
static long classify77_0(int tag, long a, long b) {
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
static long accum77_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard77_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S77_1;

static S77_1 mk77_1(long a) {
    S77_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe77_1(const S77_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read77_1(const S77_1 *s) {
    return s->a * 4;
}
static void bump77_1(S77_1 *s, long d) {
    s->a = s->a + d;
}
static long classify77_1(int tag, long a, long b) {
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
static long accum77_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard77_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S77_2;

static S77_2 mk77_2(long a) {
    S77_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe77_2(const S77_2 *s) {
    return s->a + s->n0;
}
static long read77_2(const S77_2 *s) {
    return s->a * 2;
}
static void bump77_2(S77_2 *s, long d) {
    s->a = s->a + d;
}
static long classify77_2(int tag, long a, long b) {
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
static long accum77_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard77_2(long x) {
    return x + 6;
}

long f077(long x) {
    long acc = x;
    acc += f041(x + 1);
    acc += f047(x + 2);
    S77_0 s0 = mk77_0(acc);
    bump77_0(&s0, 6);
    acc += probe77_0(&s0);
    acc += read77_0(&s0);
    acc += classify77_0(1, acc, acc);
    acc += accum77_0(6);
    acc += guard77_0(acc);
    S77_1 s1 = mk77_1(acc);
    bump77_1(&s1, 4);
    acc += probe77_1(&s1);
    acc += read77_1(&s1);
    acc += classify77_1(1, acc, acc);
    acc += accum77_1(9);
    acc += guard77_1(acc);
    S77_2 s2 = mk77_2(acc);
    bump77_2(&s2, 5);
    acc += probe77_2(&s2);
    acc += read77_2(&s2);
    acc += classify77_2(1, acc, acc);
    acc += accum77_2(3);
    acc += guard77_2(acc);
    return clampi(acc);
}
