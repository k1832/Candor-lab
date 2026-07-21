/* GENERATED C mirror of reference module m146. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S146_0;

static S146_0 mk146_0(long a) {
    S146_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe146_0(const S146_0 *s) {
    return s->a + s->n0;
}
static long read146_0(const S146_0 *s) {
    return s->a * 4;
}
static void bump146_0(S146_0 *s, long d) {
    s->a = s->a + d;
}
static long classify146_0(int tag, long a, long b) {
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
static long accum146_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard146_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S146_1;

static S146_1 mk146_1(long a) {
    S146_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe146_1(const S146_1 *s) {
    return s->a + s->n0;
}
static long read146_1(const S146_1 *s) {
    return s->a * 4;
}
static void bump146_1(S146_1 *s, long d) {
    s->a = s->a + d;
}
static long classify146_1(int tag, long a, long b) {
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
static long accum146_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard146_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S146_2;

static S146_2 mk146_2(long a) {
    S146_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe146_2(const S146_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read146_2(const S146_2 *s) {
    return s->a * 4;
}
static void bump146_2(S146_2 *s, long d) {
    s->a = s->a + d;
}
static long classify146_2(int tag, long a, long b) {
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
static long accum146_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard146_2(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S146_3;

static S146_3 mk146_3(long a) {
    S146_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe146_3(const S146_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read146_3(const S146_3 *s) {
    return s->a * 5;
}
static void bump146_3(S146_3 *s, long d) {
    s->a = s->a + d;
}
static long classify146_3(int tag, long a, long b) {
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
static long accum146_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard146_3(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S146_4;

static S146_4 mk146_4(long a) {
    S146_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe146_4(const S146_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read146_4(const S146_4 *s) {
    return s->a * 3;
}
static void bump146_4(S146_4 *s, long d) {
    s->a = s->a + d;
}
static long classify146_4(int tag, long a, long b) {
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
static long accum146_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard146_4(long x) {
    return x + 8;
}

long f146(long x) {
    long acc = x;
    acc += f026(x + 1);
    acc += f033(x + 2);
    acc += f086(x + 3);
    acc += f092(x + 4);
    S146_0 s0 = mk146_0(acc);
    bump146_0(&s0, 4);
    acc += probe146_0(&s0);
    acc += read146_0(&s0);
    acc += classify146_0(1, acc, acc);
    acc += accum146_0(8);
    acc += guard146_0(acc);
    S146_1 s1 = mk146_1(acc);
    bump146_1(&s1, 2);
    acc += probe146_1(&s1);
    acc += read146_1(&s1);
    acc += classify146_1(1, acc, acc);
    acc += accum146_1(4);
    acc += guard146_1(acc);
    S146_2 s2 = mk146_2(acc);
    bump146_2(&s2, 2);
    acc += probe146_2(&s2);
    acc += read146_2(&s2);
    acc += classify146_2(1, acc, acc);
    acc += accum146_2(7);
    acc += guard146_2(acc);
    S146_3 s3 = mk146_3(acc);
    bump146_3(&s3, 1);
    acc += probe146_3(&s3);
    acc += read146_3(&s3);
    acc += classify146_3(1, acc, acc);
    acc += accum146_3(5);
    acc += guard146_3(acc);
    S146_4 s4 = mk146_4(acc);
    bump146_4(&s4, 9);
    acc += probe146_4(&s4);
    acc += read146_4(&s4);
    acc += classify146_4(1, acc, acc);
    acc += accum146_4(8);
    acc += guard146_4(acc);
    return clampi(acc);
}
