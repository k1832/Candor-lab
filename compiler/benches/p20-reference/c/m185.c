/* GENERATED C mirror of reference module m185. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S185_0;

static S185_0 mk185_0(long a) {
    S185_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe185_0(const S185_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read185_0(const S185_0 *s) {
    return s->a * 5;
}
static void bump185_0(S185_0 *s, long d) {
    s->a = s->a + d;
}
static long classify185_0(int tag, long a, long b) {
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
static long accum185_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard185_0(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S185_1;

static S185_1 mk185_1(long a) {
    S185_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe185_1(const S185_1 *s) {
    return s->a + s->n0;
}
static long read185_1(const S185_1 *s) {
    return s->a * 4;
}
static void bump185_1(S185_1 *s, long d) {
    s->a = s->a + d;
}
static long classify185_1(int tag, long a, long b) {
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
static long accum185_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard185_1(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S185_2;

static S185_2 mk185_2(long a) {
    S185_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe185_2(const S185_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read185_2(const S185_2 *s) {
    return s->a * 2;
}
static void bump185_2(S185_2 *s, long d) {
    s->a = s->a + d;
}
static long classify185_2(int tag, long a, long b) {
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
static long accum185_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard185_2(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S185_3;

static S185_3 mk185_3(long a) {
    S185_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe185_3(const S185_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read185_3(const S185_3 *s) {
    return s->a * 2;
}
static void bump185_3(S185_3 *s, long d) {
    s->a = s->a + d;
}
static long classify185_3(int tag, long a, long b) {
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
static long accum185_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard185_3(long x) {
    return x + 8;
}

long f185(long x) {
    long acc = x;
    acc += f109(x + 1);
    acc += f120(x + 2);
    acc += f141(x + 3);
    acc += f155(x + 4);
    S185_0 s0 = mk185_0(acc);
    bump185_0(&s0, 5);
    acc += probe185_0(&s0);
    acc += read185_0(&s0);
    acc += classify185_0(1, acc, acc);
    acc += accum185_0(8);
    acc += guard185_0(acc);
    S185_1 s1 = mk185_1(acc);
    bump185_1(&s1, 4);
    acc += probe185_1(&s1);
    acc += read185_1(&s1);
    acc += classify185_1(1, acc, acc);
    acc += accum185_1(9);
    acc += guard185_1(acc);
    S185_2 s2 = mk185_2(acc);
    bump185_2(&s2, 6);
    acc += probe185_2(&s2);
    acc += read185_2(&s2);
    acc += classify185_2(1, acc, acc);
    acc += accum185_2(7);
    acc += guard185_2(acc);
    S185_3 s3 = mk185_3(acc);
    bump185_3(&s3, 9);
    acc += probe185_3(&s3);
    acc += read185_3(&s3);
    acc += classify185_3(1, acc, acc);
    acc += accum185_3(8);
    acc += guard185_3(acc);
    return clampi(acc);
}
