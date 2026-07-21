/* GENERATED C mirror of reference module m187. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S187_0;

static S187_0 mk187_0(long a) {
    S187_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe187_0(const S187_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read187_0(const S187_0 *s) {
    return s->a * 4;
}
static void bump187_0(S187_0 *s, long d) {
    s->a = s->a + d;
}
static long classify187_0(int tag, long a, long b) {
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
static long accum187_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard187_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S187_1;

static S187_1 mk187_1(long a) {
    S187_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe187_1(const S187_1 *s) {
    return s->a + s->n0;
}
static long read187_1(const S187_1 *s) {
    return s->a * 2;
}
static void bump187_1(S187_1 *s, long d) {
    s->a = s->a + d;
}
static long classify187_1(int tag, long a, long b) {
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
static long accum187_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard187_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S187_2;

static S187_2 mk187_2(long a) {
    S187_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe187_2(const S187_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read187_2(const S187_2 *s) {
    return s->a * 2;
}
static void bump187_2(S187_2 *s, long d) {
    s->a = s->a + d;
}
static long classify187_2(int tag, long a, long b) {
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
static long accum187_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard187_2(long x) {
    return x + 8;
}

long f187(long x) {
    long acc = x;
    acc += f072(x + 1);
    acc += f106(x + 2);
    acc += f146(x + 3);
    S187_0 s0 = mk187_0(acc);
    bump187_0(&s0, 9);
    acc += probe187_0(&s0);
    acc += read187_0(&s0);
    acc += classify187_0(1, acc, acc);
    acc += accum187_0(3);
    acc += guard187_0(acc);
    S187_1 s1 = mk187_1(acc);
    bump187_1(&s1, 5);
    acc += probe187_1(&s1);
    acc += read187_1(&s1);
    acc += classify187_1(1, acc, acc);
    acc += accum187_1(7);
    acc += guard187_1(acc);
    S187_2 s2 = mk187_2(acc);
    bump187_2(&s2, 5);
    acc += probe187_2(&s2);
    acc += read187_2(&s2);
    acc += classify187_2(1, acc, acc);
    acc += accum187_2(7);
    acc += guard187_2(acc);
    return clampi(acc);
}
