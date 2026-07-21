/* GENERATED C mirror of reference module m191. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S191_0;

static S191_0 mk191_0(long a) {
    S191_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe191_0(const S191_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read191_0(const S191_0 *s) {
    return s->a * 2;
}
static void bump191_0(S191_0 *s, long d) {
    s->a = s->a + d;
}
static long classify191_0(int tag, long a, long b) {
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
static long accum191_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard191_0(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S191_1;

static S191_1 mk191_1(long a) {
    S191_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe191_1(const S191_1 *s) {
    return s->a + s->n0;
}
static long read191_1(const S191_1 *s) {
    return s->a * 4;
}
static void bump191_1(S191_1 *s, long d) {
    s->a = s->a + d;
}
static long classify191_1(int tag, long a, long b) {
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
static long accum191_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard191_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S191_2;

static S191_2 mk191_2(long a) {
    S191_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe191_2(const S191_2 *s) {
    return s->a + s->n0;
}
static long read191_2(const S191_2 *s) {
    return s->a * 3;
}
static void bump191_2(S191_2 *s, long d) {
    s->a = s->a + d;
}
static long classify191_2(int tag, long a, long b) {
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
static long accum191_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard191_2(long x) {
    return x + 2;
}

long f191(long x) {
    long acc = x;
    acc += f163(x + 1);
    S191_0 s0 = mk191_0(acc);
    bump191_0(&s0, 9);
    acc += probe191_0(&s0);
    acc += read191_0(&s0);
    acc += classify191_0(1, acc, acc);
    acc += accum191_0(3);
    acc += guard191_0(acc);
    S191_1 s1 = mk191_1(acc);
    bump191_1(&s1, 9);
    acc += probe191_1(&s1);
    acc += read191_1(&s1);
    acc += classify191_1(1, acc, acc);
    acc += accum191_1(5);
    acc += guard191_1(acc);
    S191_2 s2 = mk191_2(acc);
    bump191_2(&s2, 1);
    acc += probe191_2(&s2);
    acc += read191_2(&s2);
    acc += classify191_2(1, acc, acc);
    acc += accum191_2(8);
    acc += guard191_2(acc);
    return clampi(acc);
}
