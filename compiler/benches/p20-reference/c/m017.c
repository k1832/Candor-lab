/* GENERATED C mirror of reference module m017. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S17_0;

static S17_0 mk17_0(long a) {
    S17_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe17_0(const S17_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read17_0(const S17_0 *s) {
    return s->a * 4;
}
static void bump17_0(S17_0 *s, long d) {
    s->a = s->a + d;
}
static long classify17_0(int tag, long a, long b) {
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
static long accum17_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard17_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S17_1;

static S17_1 mk17_1(long a) {
    S17_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe17_1(const S17_1 *s) {
    return s->a + s->n0;
}
static long read17_1(const S17_1 *s) {
    return s->a * 2;
}
static void bump17_1(S17_1 *s, long d) {
    s->a = s->a + d;
}
static long classify17_1(int tag, long a, long b) {
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
static long accum17_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard17_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S17_2;

static S17_2 mk17_2(long a) {
    S17_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe17_2(const S17_2 *s) {
    return s->a + s->n0;
}
static long read17_2(const S17_2 *s) {
    return s->a * 6;
}
static void bump17_2(S17_2 *s, long d) {
    s->a = s->a + d;
}
static long classify17_2(int tag, long a, long b) {
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
static long accum17_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard17_2(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S17_3;

static S17_3 mk17_3(long a) {
    S17_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe17_3(const S17_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read17_3(const S17_3 *s) {
    return s->a * 6;
}
static void bump17_3(S17_3 *s, long d) {
    s->a = s->a + d;
}
static long classify17_3(int tag, long a, long b) {
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
static long accum17_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard17_3(long x) {
    return x + 9;
}

long f017(long x) {
    long acc = x;
    acc += f003(x + 1);
    acc += f005(x + 2);
    acc += f006(x + 3);
    S17_0 s0 = mk17_0(acc);
    bump17_0(&s0, 5);
    acc += probe17_0(&s0);
    acc += read17_0(&s0);
    acc += classify17_0(1, acc, acc);
    acc += accum17_0(6);
    acc += guard17_0(acc);
    S17_1 s1 = mk17_1(acc);
    bump17_1(&s1, 4);
    acc += probe17_1(&s1);
    acc += read17_1(&s1);
    acc += classify17_1(1, acc, acc);
    acc += accum17_1(5);
    acc += guard17_1(acc);
    S17_2 s2 = mk17_2(acc);
    bump17_2(&s2, 4);
    acc += probe17_2(&s2);
    acc += read17_2(&s2);
    acc += classify17_2(1, acc, acc);
    acc += accum17_2(4);
    acc += guard17_2(acc);
    S17_3 s3 = mk17_3(acc);
    bump17_3(&s3, 8);
    acc += probe17_3(&s3);
    acc += read17_3(&s3);
    acc += classify17_3(1, acc, acc);
    acc += accum17_3(5);
    acc += guard17_3(acc);
    return clampi(acc);
}
