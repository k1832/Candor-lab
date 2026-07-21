/* GENERATED C mirror of reference module m073. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S73_0;

static S73_0 mk73_0(long a) {
    S73_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe73_0(const S73_0 *s) {
    return s->a + s->n0;
}
static long read73_0(const S73_0 *s) {
    return s->a * 3;
}
static void bump73_0(S73_0 *s, long d) {
    s->a = s->a + d;
}
static long classify73_0(int tag, long a, long b) {
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
static long accum73_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard73_0(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S73_1;

static S73_1 mk73_1(long a) {
    S73_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe73_1(const S73_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read73_1(const S73_1 *s) {
    return s->a * 7;
}
static void bump73_1(S73_1 *s, long d) {
    s->a = s->a + d;
}
static long classify73_1(int tag, long a, long b) {
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
static long accum73_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard73_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S73_2;

static S73_2 mk73_2(long a) {
    S73_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe73_2(const S73_2 *s) {
    return s->a + s->n0;
}
static long read73_2(const S73_2 *s) {
    return s->a * 6;
}
static void bump73_2(S73_2 *s, long d) {
    s->a = s->a + d;
}
static long classify73_2(int tag, long a, long b) {
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
static long accum73_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard73_2(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S73_3;

static S73_3 mk73_3(long a) {
    S73_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe73_3(const S73_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read73_3(const S73_3 *s) {
    return s->a * 4;
}
static void bump73_3(S73_3 *s, long d) {
    s->a = s->a + d;
}
static long classify73_3(int tag, long a, long b) {
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
static long accum73_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard73_3(long x) {
    return x + 8;
}

long f073(long x) {
    long acc = x;
    acc += f027(x + 1);
    S73_0 s0 = mk73_0(acc);
    bump73_0(&s0, 3);
    acc += probe73_0(&s0);
    acc += read73_0(&s0);
    acc += classify73_0(1, acc, acc);
    acc += accum73_0(8);
    acc += guard73_0(acc);
    S73_1 s1 = mk73_1(acc);
    bump73_1(&s1, 4);
    acc += probe73_1(&s1);
    acc += read73_1(&s1);
    acc += classify73_1(1, acc, acc);
    acc += accum73_1(4);
    acc += guard73_1(acc);
    S73_2 s2 = mk73_2(acc);
    bump73_2(&s2, 7);
    acc += probe73_2(&s2);
    acc += read73_2(&s2);
    acc += classify73_2(1, acc, acc);
    acc += accum73_2(4);
    acc += guard73_2(acc);
    S73_3 s3 = mk73_3(acc);
    bump73_3(&s3, 5);
    acc += probe73_3(&s3);
    acc += read73_3(&s3);
    acc += classify73_3(1, acc, acc);
    acc += accum73_3(5);
    acc += guard73_3(acc);
    return clampi(acc);
}
