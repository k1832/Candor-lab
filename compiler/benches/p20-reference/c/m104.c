/* GENERATED C mirror of reference module m104. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S104_0;

static S104_0 mk104_0(long a) {
    S104_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe104_0(const S104_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read104_0(const S104_0 *s) {
    return s->a * 3;
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
        acc += i * 3;
    }
    return acc;
}
static long guard104_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S104_1;

static S104_1 mk104_1(long a) {
    S104_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe104_1(const S104_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read104_1(const S104_1 *s) {
    return s->a * 3;
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
        acc += i * 3;
    }
    return acc;
}
static long guard104_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S104_2;

static S104_2 mk104_2(long a) {
    S104_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe104_2(const S104_2 *s) {
    return s->a + s->n0 + s->n1;
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
        acc += i * 5;
    }
    return acc;
}
static long guard104_2(long x) {
    return x + 1;
}

long f104(long x) {
    long acc = x;
    acc += f038(x + 1);
    acc += f049(x + 2);
    acc += f073(x + 3);
    acc += f074(x + 4);
    S104_0 s0 = mk104_0(acc);
    bump104_0(&s0, 3);
    acc += probe104_0(&s0);
    acc += read104_0(&s0);
    acc += classify104_0(1, acc, acc);
    acc += accum104_0(9);
    acc += guard104_0(acc);
    S104_1 s1 = mk104_1(acc);
    bump104_1(&s1, 5);
    acc += probe104_1(&s1);
    acc += read104_1(&s1);
    acc += classify104_1(1, acc, acc);
    acc += accum104_1(9);
    acc += guard104_1(acc);
    S104_2 s2 = mk104_2(acc);
    bump104_2(&s2, 9);
    acc += probe104_2(&s2);
    acc += read104_2(&s2);
    acc += classify104_2(1, acc, acc);
    acc += accum104_2(6);
    acc += guard104_2(acc);
    return clampi(acc);
}
