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
    return s->a * 5;
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
        acc += i * 2;
    }
    return acc;
}
static long guard191_0(long x) {
    return x + 5;
}

static long pick191_0_0(long a, long b) { return a > b ? a : b; }
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
    return s->a * 6;
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
        acc += i * 4;
    }
    return acc;
}
static long guard191_1(long x) {
    return x + 2;
}

static long pick191_1_0(long a, long b) { return a > b ? a : b; }
static long pick191_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S191_2;

static S191_2 mk191_2(long a) {
    S191_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe191_2(const S191_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read191_2(const S191_2 *s) {
    return s->a * 7;
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
        acc += i * 3;
    }
    return acc;
}
static long guard191_2(long x) {
    return x + 1;
}

static long pick191_2_0(long a, long b) { return a > b ? a : b; }
long f191(long x) {
    long acc = x;
    acc += f162(x + 1);
    acc += f164(x + 2);
    acc += f180(x + 3);
    S191_0 s0 = mk191_0(acc);
    bump191_0(&s0, 7);
    acc += probe191_0(&s0);
    acc += read191_0(&s0);
    acc += classify191_0(1, acc, acc);
    acc += accum191_0(3);
    acc += guard191_0(acc);
    acc += pick191_0_0(acc, acc + 2);
    S191_1 s1 = mk191_1(acc);
    bump191_1(&s1, 5);
    acc += probe191_1(&s1);
    acc += read191_1(&s1);
    acc += classify191_1(1, acc, acc);
    acc += accum191_1(8);
    acc += guard191_1(acc);
    acc += pick191_1_0(acc, acc + 4);
    acc += pick191_1_1(acc, acc + 3);
    S191_2 s2 = mk191_2(acc);
    bump191_2(&s2, 3);
    acc += probe191_2(&s2);
    acc += read191_2(&s2);
    acc += classify191_2(1, acc, acc);
    acc += accum191_2(3);
    acc += guard191_2(acc);
    acc += pick191_2_0(acc, acc + 4);
    return clampi(acc);
}
