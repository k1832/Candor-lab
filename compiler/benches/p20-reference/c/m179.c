/* GENERATED C mirror of reference module m179. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S179_0;

static S179_0 mk179_0(long a) {
    S179_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe179_0(const S179_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read179_0(const S179_0 *s) {
    return s->a * 5;
}
static void bump179_0(S179_0 *s, long d) {
    s->a = s->a + d;
}
static long classify179_0(int tag, long a, long b) {
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
static long accum179_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard179_0(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S179_1;

static S179_1 mk179_1(long a) {
    S179_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe179_1(const S179_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read179_1(const S179_1 *s) {
    return s->a * 6;
}
static void bump179_1(S179_1 *s, long d) {
    s->a = s->a + d;
}
static long classify179_1(int tag, long a, long b) {
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
static long accum179_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard179_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S179_2;

static S179_2 mk179_2(long a) {
    S179_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe179_2(const S179_2 *s) {
    return s->a + s->n0;
}
static long read179_2(const S179_2 *s) {
    return s->a * 6;
}
static void bump179_2(S179_2 *s, long d) {
    s->a = s->a + d;
}
static long classify179_2(int tag, long a, long b) {
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
static long accum179_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard179_2(long x) {
    return x + 4;
}

long f179(long x) {
    long acc = x;
    acc += f019(x + 1);
    acc += f069(x + 2);
    acc += f149(x + 3);
    S179_0 s0 = mk179_0(acc);
    bump179_0(&s0, 5);
    acc += probe179_0(&s0);
    acc += read179_0(&s0);
    acc += classify179_0(1, acc, acc);
    acc += accum179_0(9);
    acc += guard179_0(acc);
    S179_1 s1 = mk179_1(acc);
    bump179_1(&s1, 4);
    acc += probe179_1(&s1);
    acc += read179_1(&s1);
    acc += classify179_1(1, acc, acc);
    acc += accum179_1(3);
    acc += guard179_1(acc);
    S179_2 s2 = mk179_2(acc);
    bump179_2(&s2, 4);
    acc += probe179_2(&s2);
    acc += read179_2(&s2);
    acc += classify179_2(1, acc, acc);
    acc += accum179_2(7);
    acc += guard179_2(acc);
    return clampi(acc);
}
