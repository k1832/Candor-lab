/* GENERATED C mirror of reference module m198. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S198_0;

static S198_0 mk198_0(long a) {
    S198_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe198_0(const S198_0 *s) {
    return s->a + s->n0;
}
static long read198_0(const S198_0 *s) {
    return s->a * 2;
}
static void bump198_0(S198_0 *s, long d) {
    s->a = s->a + d;
}
static long classify198_0(int tag, long a, long b) {
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
static long accum198_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard198_0(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S198_1;

static S198_1 mk198_1(long a) {
    S198_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe198_1(const S198_1 *s) {
    return s->a + s->n0;
}
static long read198_1(const S198_1 *s) {
    return s->a * 3;
}
static void bump198_1(S198_1 *s, long d) {
    s->a = s->a + d;
}
static long classify198_1(int tag, long a, long b) {
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
static long accum198_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard198_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S198_2;

static S198_2 mk198_2(long a) {
    S198_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe198_2(const S198_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read198_2(const S198_2 *s) {
    return s->a * 3;
}
static void bump198_2(S198_2 *s, long d) {
    s->a = s->a + d;
}
static long classify198_2(int tag, long a, long b) {
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
static long accum198_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard198_2(long x) {
    return x + 1;
}

long f198(long x) {
    long acc = x;
    acc += f035(x + 1);
    acc += f054(x + 2);
    acc += f082(x + 3);
    acc += f174(x + 4);
    S198_0 s0 = mk198_0(acc);
    bump198_0(&s0, 4);
    acc += probe198_0(&s0);
    acc += read198_0(&s0);
    acc += classify198_0(1, acc, acc);
    acc += accum198_0(7);
    acc += guard198_0(acc);
    S198_1 s1 = mk198_1(acc);
    bump198_1(&s1, 9);
    acc += probe198_1(&s1);
    acc += read198_1(&s1);
    acc += classify198_1(1, acc, acc);
    acc += accum198_1(8);
    acc += guard198_1(acc);
    S198_2 s2 = mk198_2(acc);
    bump198_2(&s2, 3);
    acc += probe198_2(&s2);
    acc += read198_2(&s2);
    acc += classify198_2(1, acc, acc);
    acc += accum198_2(3);
    acc += guard198_2(acc);
    return clampi(acc);
}
