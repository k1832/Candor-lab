/* GENERATED C mirror of reference module m182. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S182_0;

static S182_0 mk182_0(long a) {
    S182_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe182_0(const S182_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read182_0(const S182_0 *s) {
    return s->a * 5;
}
static void bump182_0(S182_0 *s, long d) {
    s->a = s->a + d;
}
static long classify182_0(int tag, long a, long b) {
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
static long accum182_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard182_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S182_1;

static S182_1 mk182_1(long a) {
    S182_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe182_1(const S182_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read182_1(const S182_1 *s) {
    return s->a * 3;
}
static void bump182_1(S182_1 *s, long d) {
    s->a = s->a + d;
}
static long classify182_1(int tag, long a, long b) {
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
static long accum182_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard182_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S182_2;

static S182_2 mk182_2(long a) {
    S182_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe182_2(const S182_2 *s) {
    return s->a + s->n0;
}
static long read182_2(const S182_2 *s) {
    return s->a * 6;
}
static void bump182_2(S182_2 *s, long d) {
    s->a = s->a + d;
}
static long classify182_2(int tag, long a, long b) {
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
static long accum182_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard182_2(long x) {
    return x + 8;
}

long f182(long x) {
    long acc = x;
    acc += f076(x + 1);
    S182_0 s0 = mk182_0(acc);
    bump182_0(&s0, 1);
    acc += probe182_0(&s0);
    acc += read182_0(&s0);
    acc += classify182_0(1, acc, acc);
    acc += accum182_0(7);
    acc += guard182_0(acc);
    S182_1 s1 = mk182_1(acc);
    bump182_1(&s1, 3);
    acc += probe182_1(&s1);
    acc += read182_1(&s1);
    acc += classify182_1(1, acc, acc);
    acc += accum182_1(9);
    acc += guard182_1(acc);
    S182_2 s2 = mk182_2(acc);
    bump182_2(&s2, 3);
    acc += probe182_2(&s2);
    acc += read182_2(&s2);
    acc += classify182_2(1, acc, acc);
    acc += accum182_2(8);
    acc += guard182_2(acc);
    return clampi(acc);
}
