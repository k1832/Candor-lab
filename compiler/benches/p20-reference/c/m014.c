/* GENERATED C mirror of reference module m014. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S14_0;

static S14_0 mk14_0(long a) {
    S14_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe14_0(const S14_0 *s) {
    return s->a + s->n0;
}
static long read14_0(const S14_0 *s) {
    return s->a * 7;
}
static void bump14_0(S14_0 *s, long d) {
    s->a = s->a + d;
}
static long classify14_0(int tag, long a, long b) {
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
static long accum14_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard14_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S14_1;

static S14_1 mk14_1(long a) {
    S14_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe14_1(const S14_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read14_1(const S14_1 *s) {
    return s->a * 6;
}
static void bump14_1(S14_1 *s, long d) {
    s->a = s->a + d;
}
static long classify14_1(int tag, long a, long b) {
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
static long accum14_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard14_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S14_2;

static S14_2 mk14_2(long a) {
    S14_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe14_2(const S14_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read14_2(const S14_2 *s) {
    return s->a * 4;
}
static void bump14_2(S14_2 *s, long d) {
    s->a = s->a + d;
}
static long classify14_2(int tag, long a, long b) {
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
static long accum14_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard14_2(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S14_3;

static S14_3 mk14_3(long a) {
    S14_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe14_3(const S14_3 *s) {
    return s->a + s->n0;
}
static long read14_3(const S14_3 *s) {
    return s->a * 4;
}
static void bump14_3(S14_3 *s, long d) {
    s->a = s->a + d;
}
static long classify14_3(int tag, long a, long b) {
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
static long accum14_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard14_3(long x) {
    return x + 8;
}

long f014(long x) {
    long acc = x;
    acc += f000(x + 1);
    acc += f001(x + 2);
    S14_0 s0 = mk14_0(acc);
    bump14_0(&s0, 4);
    acc += probe14_0(&s0);
    acc += read14_0(&s0);
    acc += classify14_0(1, acc, acc);
    acc += accum14_0(8);
    acc += guard14_0(acc);
    S14_1 s1 = mk14_1(acc);
    bump14_1(&s1, 2);
    acc += probe14_1(&s1);
    acc += read14_1(&s1);
    acc += classify14_1(1, acc, acc);
    acc += accum14_1(8);
    acc += guard14_1(acc);
    S14_2 s2 = mk14_2(acc);
    bump14_2(&s2, 8);
    acc += probe14_2(&s2);
    acc += read14_2(&s2);
    acc += classify14_2(1, acc, acc);
    acc += accum14_2(4);
    acc += guard14_2(acc);
    S14_3 s3 = mk14_3(acc);
    bump14_3(&s3, 9);
    acc += probe14_3(&s3);
    acc += read14_3(&s3);
    acc += classify14_3(1, acc, acc);
    acc += accum14_3(4);
    acc += guard14_3(acc);
    return clampi(acc);
}
