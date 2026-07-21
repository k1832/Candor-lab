/* GENERATED C mirror of reference module m162. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S162_0;

static S162_0 mk162_0(long a) {
    S162_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe162_0(const S162_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read162_0(const S162_0 *s) {
    return s->a * 4;
}
static void bump162_0(S162_0 *s, long d) {
    s->a = s->a + d;
}
static long classify162_0(int tag, long a, long b) {
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
static long accum162_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard162_0(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S162_1;

static S162_1 mk162_1(long a) {
    S162_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe162_1(const S162_1 *s) {
    return s->a + s->n0;
}
static long read162_1(const S162_1 *s) {
    return s->a * 7;
}
static void bump162_1(S162_1 *s, long d) {
    s->a = s->a + d;
}
static long classify162_1(int tag, long a, long b) {
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
static long accum162_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard162_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S162_2;

static S162_2 mk162_2(long a) {
    S162_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe162_2(const S162_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read162_2(const S162_2 *s) {
    return s->a * 6;
}
static void bump162_2(S162_2 *s, long d) {
    s->a = s->a + d;
}
static long classify162_2(int tag, long a, long b) {
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
static long accum162_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard162_2(long x) {
    return x + 1;
}

long f162(long x) {
    long acc = x;
    acc += f048(x + 1);
    acc += f107(x + 2);
    S162_0 s0 = mk162_0(acc);
    bump162_0(&s0, 9);
    acc += probe162_0(&s0);
    acc += read162_0(&s0);
    acc += classify162_0(1, acc, acc);
    acc += accum162_0(7);
    acc += guard162_0(acc);
    S162_1 s1 = mk162_1(acc);
    bump162_1(&s1, 6);
    acc += probe162_1(&s1);
    acc += read162_1(&s1);
    acc += classify162_1(1, acc, acc);
    acc += accum162_1(3);
    acc += guard162_1(acc);
    S162_2 s2 = mk162_2(acc);
    bump162_2(&s2, 6);
    acc += probe162_2(&s2);
    acc += read162_2(&s2);
    acc += classify162_2(1, acc, acc);
    acc += accum162_2(7);
    acc += guard162_2(acc);
    return clampi(acc);
}
