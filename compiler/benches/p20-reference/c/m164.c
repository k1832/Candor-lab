/* GENERATED C mirror of reference module m164. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S164_0;

static S164_0 mk164_0(long a) {
    S164_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe164_0(const S164_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read164_0(const S164_0 *s) {
    return s->a * 6;
}
static void bump164_0(S164_0 *s, long d) {
    s->a = s->a + d;
}
static long classify164_0(int tag, long a, long b) {
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
static long accum164_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard164_0(long x) {
    return x + 4;
}

static long pick164_0_0(long a, long b) { return a > b ? a : b; }
static long pick164_0_1(long a, long b) { return a > b ? a : b; }
static long pick164_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S164_1;

static S164_1 mk164_1(long a) {
    S164_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe164_1(const S164_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read164_1(const S164_1 *s) {
    return s->a * 2;
}
static void bump164_1(S164_1 *s, long d) {
    s->a = s->a + d;
}
static long classify164_1(int tag, long a, long b) {
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
static long accum164_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard164_1(long x) {
    return x + 5;
}

static long pick164_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S164_2;

static S164_2 mk164_2(long a) {
    S164_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe164_2(const S164_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read164_2(const S164_2 *s) {
    return s->a * 6;
}
static void bump164_2(S164_2 *s, long d) {
    s->a = s->a + d;
}
static long classify164_2(int tag, long a, long b) {
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
static long accum164_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard164_2(long x) {
    return x + 5;
}

static long pick164_2_0(long a, long b) { return a > b ? a : b; }
long f164(long x) {
    long acc = x;
    acc += f011(x + 1);
    acc += f101(x + 2);
    acc += f127(x + 3);
    S164_0 s0 = mk164_0(acc);
    bump164_0(&s0, 3);
    acc += probe164_0(&s0);
    acc += read164_0(&s0);
    acc += classify164_0(1, acc, acc);
    acc += accum164_0(5);
    acc += guard164_0(acc);
    acc += pick164_0_0(acc, acc + 7);
    acc += pick164_0_1(acc, acc + 2);
    acc += pick164_0_2(acc, acc + 1);
    S164_1 s1 = mk164_1(acc);
    bump164_1(&s1, 7);
    acc += probe164_1(&s1);
    acc += read164_1(&s1);
    acc += classify164_1(1, acc, acc);
    acc += accum164_1(5);
    acc += guard164_1(acc);
    acc += pick164_1_0(acc, acc + 3);
    S164_2 s2 = mk164_2(acc);
    bump164_2(&s2, 8);
    acc += probe164_2(&s2);
    acc += read164_2(&s2);
    acc += classify164_2(1, acc, acc);
    acc += accum164_2(5);
    acc += guard164_2(acc);
    acc += pick164_2_0(acc, acc + 8);
    return clampi(acc);
}
