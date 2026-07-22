/* GENERATED C mirror of reference module m189. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S189_0;

static S189_0 mk189_0(long a) {
    S189_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe189_0(const S189_0 *s) {
    return s->a + s->n0;
}
static long read189_0(const S189_0 *s) {
    return s->a * 6;
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
        acc += i * 3;
    }
    return acc;
}
static long guard189_0(long x) {
    return x + 9;
}

static long pick189_0_0(long a, long b) { return a > b ? a : b; }
static long pick189_0_1(long a, long b) { return a > b ? a : b; }
static long pick189_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S189_1;

static S189_1 mk189_1(long a) {
    S189_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe189_1(const S189_1 *s) {
    return s->a + s->n0;
}
static long read189_1(const S189_1 *s) {
    return s->a * 3;
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
        acc += i * 2;
    }
    return acc;
}
static long guard189_1(long x) {
    return x + 9;
}

static long pick189_1_0(long a, long b) { return a > b ? a : b; }
static long pick189_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S189_2;

static S189_2 mk189_2(long a) {
    S189_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe189_2(const S189_2 *s) {
    return s->a + s->n0;
}
static long read189_2(const S189_2 *s) {
    return s->a * 3;
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
        acc += i * 4;
    }
    return acc;
}
static long guard189_2(long x) {
    return x + 4;
}

static long pick189_2_0(long a, long b) { return a > b ? a : b; }
static long pick189_2_1(long a, long b) { return a > b ? a : b; }
long f189(long x) {
    long acc = x;
    acc += f037(x + 1);
    acc += f113(x + 2);
    acc += f130(x + 3);
    S189_0 s0 = mk189_0(acc);
    bump189_0(&s0, 5);
    acc += probe189_0(&s0);
    acc += read189_0(&s0);
    acc += classify189_0(1, acc, acc);
    acc += accum189_0(7);
    acc += guard189_0(acc);
    acc += pick189_0_0(acc, acc + 3);
    acc += pick189_0_1(acc, acc + 4);
    acc += pick189_0_2(acc, acc + 3);
    S189_1 s1 = mk189_1(acc);
    bump189_1(&s1, 7);
    acc += probe189_1(&s1);
    acc += read189_1(&s1);
    acc += classify189_1(1, acc, acc);
    acc += accum189_1(6);
    acc += guard189_1(acc);
    acc += pick189_1_0(acc, acc + 9);
    acc += pick189_1_1(acc, acc + 9);
    S189_2 s2 = mk189_2(acc);
    bump189_2(&s2, 2);
    acc += probe189_2(&s2);
    acc += read189_2(&s2);
    acc += classify189_2(1, acc, acc);
    acc += accum189_2(8);
    acc += guard189_2(acc);
    acc += pick189_2_0(acc, acc + 2);
    acc += pick189_2_1(acc, acc + 7);
    return clampi(acc);
}
