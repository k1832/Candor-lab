/* GENERATED C mirror of reference module m009. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S9_0;

static S9_0 mk9_0(long a) {
    S9_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe9_0(const S9_0 *s) {
    return s->a + s->n0;
}
static long read9_0(const S9_0 *s) {
    return s->a * 4;
}
static void bump9_0(S9_0 *s, long d) {
    s->a = s->a + d;
}
static long classify9_0(int tag, long a, long b) {
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
static long accum9_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard9_0(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S9_1;

static S9_1 mk9_1(long a) {
    S9_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe9_1(const S9_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read9_1(const S9_1 *s) {
    return s->a * 3;
}
static void bump9_1(S9_1 *s, long d) {
    s->a = s->a + d;
}
static long classify9_1(int tag, long a, long b) {
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
static long accum9_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard9_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S9_2;

static S9_2 mk9_2(long a) {
    S9_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe9_2(const S9_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read9_2(const S9_2 *s) {
    return s->a * 2;
}
static void bump9_2(S9_2 *s, long d) {
    s->a = s->a + d;
}
static long classify9_2(int tag, long a, long b) {
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
static long accum9_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard9_2(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S9_3;

static S9_3 mk9_3(long a) {
    S9_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe9_3(const S9_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read9_3(const S9_3 *s) {
    return s->a * 5;
}
static void bump9_3(S9_3 *s, long d) {
    s->a = s->a + d;
}
static long classify9_3(int tag, long a, long b) {
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
static long accum9_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard9_3(long x) {
    return x + 1;
}

long f009(long x) {
    long acc = x;
    acc += f007(x + 1);
    S9_0 s0 = mk9_0(acc);
    bump9_0(&s0, 5);
    acc += probe9_0(&s0);
    acc += read9_0(&s0);
    acc += classify9_0(1, acc, acc);
    acc += accum9_0(6);
    acc += guard9_0(acc);
    S9_1 s1 = mk9_1(acc);
    bump9_1(&s1, 5);
    acc += probe9_1(&s1);
    acc += read9_1(&s1);
    acc += classify9_1(1, acc, acc);
    acc += accum9_1(5);
    acc += guard9_1(acc);
    S9_2 s2 = mk9_2(acc);
    bump9_2(&s2, 4);
    acc += probe9_2(&s2);
    acc += read9_2(&s2);
    acc += classify9_2(1, acc, acc);
    acc += accum9_2(9);
    acc += guard9_2(acc);
    S9_3 s3 = mk9_3(acc);
    bump9_3(&s3, 6);
    acc += probe9_3(&s3);
    acc += read9_3(&s3);
    acc += classify9_3(1, acc, acc);
    acc += accum9_3(4);
    acc += guard9_3(acc);
    return clampi(acc);
}
