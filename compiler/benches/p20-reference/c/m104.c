/* GENERATED C mirror of reference module m104. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S104_0;

static S104_0 mk104_0(long a) {
    S104_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe104_0(const S104_0 *s) {
    return s->a + s->n0;
}
static long read104_0(const S104_0 *s) {
    return s->a * 4;
}
static void bump104_0(S104_0 *s, long d) {
    s->a = s->a + d;
}
static long classify104_0(int tag, long a, long b) {
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
static long accum104_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard104_0(long x) {
    return x + 5;
}

static long pick104_0_0(long a, long b) { return a > b ? a : b; }
static long pick104_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S104_1;

static S104_1 mk104_1(long a) {
    S104_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe104_1(const S104_1 *s) {
    return s->a + s->n0;
}
static long read104_1(const S104_1 *s) {
    return s->a * 7;
}
static void bump104_1(S104_1 *s, long d) {
    s->a = s->a + d;
}
static long classify104_1(int tag, long a, long b) {
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
static long accum104_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard104_1(long x) {
    return x + 8;
}

static long pick104_1_0(long a, long b) { return a > b ? a : b; }
static long pick104_1_1(long a, long b) { return a > b ? a : b; }
static long pick104_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S104_2;

static S104_2 mk104_2(long a) {
    S104_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe104_2(const S104_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read104_2(const S104_2 *s) {
    return s->a * 4;
}
static void bump104_2(S104_2 *s, long d) {
    s->a = s->a + d;
}
static long classify104_2(int tag, long a, long b) {
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
static long accum104_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard104_2(long x) {
    return x + 4;
}

static long pick104_2_0(long a, long b) { return a > b ? a : b; }
long f104(long x) {
    long acc = x;
    acc += f021(x + 1);
    acc += f030(x + 2);
    acc += f043(x + 3);
    S104_0 s0 = mk104_0(acc);
    bump104_0(&s0, 8);
    acc += probe104_0(&s0);
    acc += read104_0(&s0);
    acc += classify104_0(1, acc, acc);
    acc += accum104_0(8);
    acc += guard104_0(acc);
    acc += pick104_0_0(acc, acc + 8);
    acc += pick104_0_1(acc, acc + 8);
    S104_1 s1 = mk104_1(acc);
    bump104_1(&s1, 8);
    acc += probe104_1(&s1);
    acc += read104_1(&s1);
    acc += classify104_1(1, acc, acc);
    acc += accum104_1(5);
    acc += guard104_1(acc);
    acc += pick104_1_0(acc, acc + 2);
    acc += pick104_1_1(acc, acc + 9);
    acc += pick104_1_2(acc, acc + 1);
    S104_2 s2 = mk104_2(acc);
    bump104_2(&s2, 9);
    acc += probe104_2(&s2);
    acc += read104_2(&s2);
    acc += classify104_2(1, acc, acc);
    acc += accum104_2(9);
    acc += guard104_2(acc);
    acc += pick104_2_0(acc, acc + 3);
    return clampi(acc);
}
