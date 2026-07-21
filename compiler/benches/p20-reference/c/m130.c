/* GENERATED C mirror of reference module m130. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S130_0;

static S130_0 mk130_0(long a) {
    S130_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe130_0(const S130_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read130_0(const S130_0 *s) {
    return s->a * 5;
}
static void bump130_0(S130_0 *s, long d) {
    s->a = s->a + d;
}
static long classify130_0(int tag, long a, long b) {
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
static long accum130_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard130_0(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S130_1;

static S130_1 mk130_1(long a) {
    S130_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe130_1(const S130_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read130_1(const S130_1 *s) {
    return s->a * 7;
}
static void bump130_1(S130_1 *s, long d) {
    s->a = s->a + d;
}
static long classify130_1(int tag, long a, long b) {
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
static long accum130_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard130_1(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S130_2;

static S130_2 mk130_2(long a) {
    S130_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe130_2(const S130_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read130_2(const S130_2 *s) {
    return s->a * 6;
}
static void bump130_2(S130_2 *s, long d) {
    s->a = s->a + d;
}
static long classify130_2(int tag, long a, long b) {
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
static long accum130_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard130_2(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S130_3;

static S130_3 mk130_3(long a) {
    S130_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe130_3(const S130_3 *s) {
    return s->a + s->n0;
}
static long read130_3(const S130_3 *s) {
    return s->a * 7;
}
static void bump130_3(S130_3 *s, long d) {
    s->a = s->a + d;
}
static long classify130_3(int tag, long a, long b) {
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
static long accum130_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard130_3(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S130_4;

static S130_4 mk130_4(long a) {
    S130_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe130_4(const S130_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read130_4(const S130_4 *s) {
    return s->a * 6;
}
static void bump130_4(S130_4 *s, long d) {
    s->a = s->a + d;
}
static long classify130_4(int tag, long a, long b) {
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
static long accum130_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard130_4(long x) {
    return x + 2;
}

long f130(long x) {
    long acc = x;
    acc += f017(x + 1);
    acc += f058(x + 2);
    acc += f089(x + 3);
    S130_0 s0 = mk130_0(acc);
    bump130_0(&s0, 4);
    acc += probe130_0(&s0);
    acc += read130_0(&s0);
    acc += classify130_0(1, acc, acc);
    acc += accum130_0(4);
    acc += guard130_0(acc);
    S130_1 s1 = mk130_1(acc);
    bump130_1(&s1, 4);
    acc += probe130_1(&s1);
    acc += read130_1(&s1);
    acc += classify130_1(1, acc, acc);
    acc += accum130_1(9);
    acc += guard130_1(acc);
    S130_2 s2 = mk130_2(acc);
    bump130_2(&s2, 4);
    acc += probe130_2(&s2);
    acc += read130_2(&s2);
    acc += classify130_2(1, acc, acc);
    acc += accum130_2(6);
    acc += guard130_2(acc);
    S130_3 s3 = mk130_3(acc);
    bump130_3(&s3, 5);
    acc += probe130_3(&s3);
    acc += read130_3(&s3);
    acc += classify130_3(1, acc, acc);
    acc += accum130_3(7);
    acc += guard130_3(acc);
    S130_4 s4 = mk130_4(acc);
    bump130_4(&s4, 9);
    acc += probe130_4(&s4);
    acc += read130_4(&s4);
    acc += classify130_4(1, acc, acc);
    acc += accum130_4(6);
    acc += guard130_4(acc);
    return clampi(acc);
}
