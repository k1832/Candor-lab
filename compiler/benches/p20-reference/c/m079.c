/* GENERATED C mirror of reference module m079. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S79_0;

static S79_0 mk79_0(long a) {
    S79_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe79_0(const S79_0 *s) {
    return s->a + s->n0;
}
static long read79_0(const S79_0 *s) {
    return s->a * 3;
}
static void bump79_0(S79_0 *s, long d) {
    s->a = s->a + d;
}
static long classify79_0(int tag, long a, long b) {
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
static long accum79_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard79_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S79_1;

static S79_1 mk79_1(long a) {
    S79_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe79_1(const S79_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read79_1(const S79_1 *s) {
    return s->a * 5;
}
static void bump79_1(S79_1 *s, long d) {
    s->a = s->a + d;
}
static long classify79_1(int tag, long a, long b) {
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
static long accum79_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard79_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S79_2;

static S79_2 mk79_2(long a) {
    S79_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe79_2(const S79_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read79_2(const S79_2 *s) {
    return s->a * 2;
}
static void bump79_2(S79_2 *s, long d) {
    s->a = s->a + d;
}
static long classify79_2(int tag, long a, long b) {
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
static long accum79_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard79_2(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S79_3;

static S79_3 mk79_3(long a) {
    S79_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe79_3(const S79_3 *s) {
    return s->a + s->n0;
}
static long read79_3(const S79_3 *s) {
    return s->a * 5;
}
static void bump79_3(S79_3 *s, long d) {
    s->a = s->a + d;
}
static long classify79_3(int tag, long a, long b) {
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
static long accum79_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard79_3(long x) {
    return x + 6;
}

long f079(long x) {
    long acc = x;
    acc += f036(x + 1);
    acc += f059(x + 2);
    S79_0 s0 = mk79_0(acc);
    bump79_0(&s0, 7);
    acc += probe79_0(&s0);
    acc += read79_0(&s0);
    acc += classify79_0(1, acc, acc);
    acc += accum79_0(9);
    acc += guard79_0(acc);
    S79_1 s1 = mk79_1(acc);
    bump79_1(&s1, 9);
    acc += probe79_1(&s1);
    acc += read79_1(&s1);
    acc += classify79_1(1, acc, acc);
    acc += accum79_1(5);
    acc += guard79_1(acc);
    S79_2 s2 = mk79_2(acc);
    bump79_2(&s2, 1);
    acc += probe79_2(&s2);
    acc += read79_2(&s2);
    acc += classify79_2(1, acc, acc);
    acc += accum79_2(4);
    acc += guard79_2(acc);
    S79_3 s3 = mk79_3(acc);
    bump79_3(&s3, 1);
    acc += probe79_3(&s3);
    acc += read79_3(&s3);
    acc += classify79_3(1, acc, acc);
    acc += accum79_3(6);
    acc += guard79_3(acc);
    return clampi(acc);
}
