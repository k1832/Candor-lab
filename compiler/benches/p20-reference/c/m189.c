/* GENERATED C mirror of reference module m189. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S189_0;

static S189_0 mk189_0(long a) {
    S189_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe189_0(const S189_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read189_0(const S189_0 *s) {
    return s->a * 3;
}
static void bump189_0(S189_0 *s, long d) {
    s->a = s->a + d;
}
static long classify189_0(int tag, long a, long b) {
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
static long accum189_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard189_0(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S189_1;

static S189_1 mk189_1(long a) {
    S189_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe189_1(const S189_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read189_1(const S189_1 *s) {
    return s->a * 6;
}
static void bump189_1(S189_1 *s, long d) {
    s->a = s->a + d;
}
static long classify189_1(int tag, long a, long b) {
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
static long accum189_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard189_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S189_2;

static S189_2 mk189_2(long a) {
    S189_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe189_2(const S189_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read189_2(const S189_2 *s) {
    return s->a * 7;
}
static void bump189_2(S189_2 *s, long d) {
    s->a = s->a + d;
}
static long classify189_2(int tag, long a, long b) {
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
static long accum189_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard189_2(long x) {
    return x + 8;
}

long f189(long x) {
    long acc = x;
    acc += f059(x + 1);
    acc += f178(x + 2);
    S189_0 s0 = mk189_0(acc);
    bump189_0(&s0, 3);
    acc += probe189_0(&s0);
    acc += read189_0(&s0);
    acc += classify189_0(1, acc, acc);
    acc += accum189_0(3);
    acc += guard189_0(acc);
    S189_1 s1 = mk189_1(acc);
    bump189_1(&s1, 6);
    acc += probe189_1(&s1);
    acc += read189_1(&s1);
    acc += classify189_1(1, acc, acc);
    acc += accum189_1(8);
    acc += guard189_1(acc);
    S189_2 s2 = mk189_2(acc);
    bump189_2(&s2, 3);
    acc += probe189_2(&s2);
    acc += read189_2(&s2);
    acc += classify189_2(1, acc, acc);
    acc += accum189_2(6);
    acc += guard189_2(acc);
    return clampi(acc);
}
