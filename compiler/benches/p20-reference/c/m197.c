/* GENERATED C mirror of reference module m197. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S197_0;

static S197_0 mk197_0(long a) {
    S197_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe197_0(const S197_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read197_0(const S197_0 *s) {
    return s->a * 7;
}
static void bump197_0(S197_0 *s, long d) {
    s->a = s->a + d;
}
static long classify197_0(int tag, long a, long b) {
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
static long accum197_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard197_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S197_1;

static S197_1 mk197_1(long a) {
    S197_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe197_1(const S197_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read197_1(const S197_1 *s) {
    return s->a * 6;
}
static void bump197_1(S197_1 *s, long d) {
    s->a = s->a + d;
}
static long classify197_1(int tag, long a, long b) {
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
static long accum197_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard197_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S197_2;

static S197_2 mk197_2(long a) {
    S197_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe197_2(const S197_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read197_2(const S197_2 *s) {
    return s->a * 4;
}
static void bump197_2(S197_2 *s, long d) {
    s->a = s->a + d;
}
static long classify197_2(int tag, long a, long b) {
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
static long accum197_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard197_2(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S197_3;

static S197_3 mk197_3(long a) {
    S197_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe197_3(const S197_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read197_3(const S197_3 *s) {
    return s->a * 6;
}
static void bump197_3(S197_3 *s, long d) {
    s->a = s->a + d;
}
static long classify197_3(int tag, long a, long b) {
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
static long accum197_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard197_3(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S197_4;

static S197_4 mk197_4(long a) {
    S197_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe197_4(const S197_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read197_4(const S197_4 *s) {
    return s->a * 7;
}
static void bump197_4(S197_4 *s, long d) {
    s->a = s->a + d;
}
static long classify197_4(int tag, long a, long b) {
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
static long accum197_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard197_4(long x) {
    return x + 1;
}

long f197(long x) {
    long acc = x;
    acc += f010(x + 1);
    S197_0 s0 = mk197_0(acc);
    bump197_0(&s0, 9);
    acc += probe197_0(&s0);
    acc += read197_0(&s0);
    acc += classify197_0(1, acc, acc);
    acc += accum197_0(8);
    acc += guard197_0(acc);
    S197_1 s1 = mk197_1(acc);
    bump197_1(&s1, 9);
    acc += probe197_1(&s1);
    acc += read197_1(&s1);
    acc += classify197_1(1, acc, acc);
    acc += accum197_1(5);
    acc += guard197_1(acc);
    S197_2 s2 = mk197_2(acc);
    bump197_2(&s2, 4);
    acc += probe197_2(&s2);
    acc += read197_2(&s2);
    acc += classify197_2(1, acc, acc);
    acc += accum197_2(9);
    acc += guard197_2(acc);
    S197_3 s3 = mk197_3(acc);
    bump197_3(&s3, 3);
    acc += probe197_3(&s3);
    acc += read197_3(&s3);
    acc += classify197_3(1, acc, acc);
    acc += accum197_3(6);
    acc += guard197_3(acc);
    S197_4 s4 = mk197_4(acc);
    bump197_4(&s4, 8);
    acc += probe197_4(&s4);
    acc += read197_4(&s4);
    acc += classify197_4(1, acc, acc);
    acc += accum197_4(3);
    acc += guard197_4(acc);
    return clampi(acc);
}
