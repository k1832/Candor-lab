/* GENERATED C mirror of reference module m137. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S137_0;

static S137_0 mk137_0(long a) {
    S137_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe137_0(const S137_0 *s) {
    return s->a + s->n0;
}
static long read137_0(const S137_0 *s) {
    return s->a * 5;
}
static void bump137_0(S137_0 *s, long d) {
    s->a = s->a + d;
}
static long classify137_0(int tag, long a, long b) {
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
static long accum137_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard137_0(long x) {
    return x + 2;
}

static long pick137_0_0(long a, long b) { return a > b ? a : b; }
static long pick137_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S137_1;

static S137_1 mk137_1(long a) {
    S137_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe137_1(const S137_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read137_1(const S137_1 *s) {
    return s->a * 3;
}
static void bump137_1(S137_1 *s, long d) {
    s->a = s->a + d;
}
static long classify137_1(int tag, long a, long b) {
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
static long accum137_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard137_1(long x) {
    return x + 2;
}

static long pick137_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S137_2;

static S137_2 mk137_2(long a) {
    S137_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe137_2(const S137_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read137_2(const S137_2 *s) {
    return s->a * 4;
}
static void bump137_2(S137_2 *s, long d) {
    s->a = s->a + d;
}
static long classify137_2(int tag, long a, long b) {
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
static long accum137_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard137_2(long x) {
    return x + 8;
}

static long pick137_2_0(long a, long b) { return a > b ? a : b; }
static long pick137_2_1(long a, long b) { return a > b ? a : b; }
static long pick137_2_2(long a, long b) { return a > b ? a : b; }
long f137(long x) {
    long acc = x;
    acc += f022(x + 1);
    acc += f023(x + 2);
    acc += f032(x + 3);
    acc += f061(x + 4);
    S137_0 s0 = mk137_0(acc);
    bump137_0(&s0, 6);
    acc += probe137_0(&s0);
    acc += read137_0(&s0);
    acc += classify137_0(1, acc, acc);
    acc += accum137_0(5);
    acc += guard137_0(acc);
    acc += pick137_0_0(acc, acc + 8);
    acc += pick137_0_1(acc, acc + 4);
    S137_1 s1 = mk137_1(acc);
    bump137_1(&s1, 8);
    acc += probe137_1(&s1);
    acc += read137_1(&s1);
    acc += classify137_1(1, acc, acc);
    acc += accum137_1(7);
    acc += guard137_1(acc);
    acc += pick137_1_0(acc, acc + 7);
    S137_2 s2 = mk137_2(acc);
    bump137_2(&s2, 5);
    acc += probe137_2(&s2);
    acc += read137_2(&s2);
    acc += classify137_2(1, acc, acc);
    acc += accum137_2(7);
    acc += guard137_2(acc);
    acc += pick137_2_0(acc, acc + 4);
    acc += pick137_2_1(acc, acc + 8);
    acc += pick137_2_2(acc, acc + 5);
    return clampi(acc);
}
