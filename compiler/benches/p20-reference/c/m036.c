/* GENERATED C mirror of reference module m036. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S36_0;

static S36_0 mk36_0(long a) {
    S36_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe36_0(const S36_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read36_0(const S36_0 *s) {
    return s->a * 6;
}
static void bump36_0(S36_0 *s, long d) {
    s->a = s->a + d;
}
static long classify36_0(int tag, long a, long b) {
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
static long accum36_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard36_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S36_1;

static S36_1 mk36_1(long a) {
    S36_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe36_1(const S36_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read36_1(const S36_1 *s) {
    return s->a * 7;
}
static void bump36_1(S36_1 *s, long d) {
    s->a = s->a + d;
}
static long classify36_1(int tag, long a, long b) {
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
static long accum36_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard36_1(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S36_2;

static S36_2 mk36_2(long a) {
    S36_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe36_2(const S36_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read36_2(const S36_2 *s) {
    return s->a * 2;
}
static void bump36_2(S36_2 *s, long d) {
    s->a = s->a + d;
}
static long classify36_2(int tag, long a, long b) {
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
static long accum36_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard36_2(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S36_3;

static S36_3 mk36_3(long a) {
    S36_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe36_3(const S36_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read36_3(const S36_3 *s) {
    return s->a * 6;
}
static void bump36_3(S36_3 *s, long d) {
    s->a = s->a + d;
}
static long classify36_3(int tag, long a, long b) {
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
static long accum36_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard36_3(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S36_4;

static S36_4 mk36_4(long a) {
    S36_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe36_4(const S36_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read36_4(const S36_4 *s) {
    return s->a * 2;
}
static void bump36_4(S36_4 *s, long d) {
    s->a = s->a + d;
}
static long classify36_4(int tag, long a, long b) {
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
static long accum36_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard36_4(long x) {
    return x + 1;
}

long f036(long x) {
    long acc = x;
    acc += f002(x + 1);
    acc += f009(x + 2);
    acc += f010(x + 3);
    S36_0 s0 = mk36_0(acc);
    bump36_0(&s0, 4);
    acc += probe36_0(&s0);
    acc += read36_0(&s0);
    acc += classify36_0(1, acc, acc);
    acc += accum36_0(3);
    acc += guard36_0(acc);
    S36_1 s1 = mk36_1(acc);
    bump36_1(&s1, 2);
    acc += probe36_1(&s1);
    acc += read36_1(&s1);
    acc += classify36_1(1, acc, acc);
    acc += accum36_1(4);
    acc += guard36_1(acc);
    S36_2 s2 = mk36_2(acc);
    bump36_2(&s2, 5);
    acc += probe36_2(&s2);
    acc += read36_2(&s2);
    acc += classify36_2(1, acc, acc);
    acc += accum36_2(8);
    acc += guard36_2(acc);
    S36_3 s3 = mk36_3(acc);
    bump36_3(&s3, 3);
    acc += probe36_3(&s3);
    acc += read36_3(&s3);
    acc += classify36_3(1, acc, acc);
    acc += accum36_3(9);
    acc += guard36_3(acc);
    S36_4 s4 = mk36_4(acc);
    bump36_4(&s4, 7);
    acc += probe36_4(&s4);
    acc += read36_4(&s4);
    acc += classify36_4(1, acc, acc);
    acc += accum36_4(3);
    acc += guard36_4(acc);
    return clampi(acc);
}
