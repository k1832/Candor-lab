/* GENERATED C mirror of reference module m070. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S70_0;

static S70_0 mk70_0(long a) {
    S70_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe70_0(const S70_0 *s) {
    return s->a + s->n0;
}
static long read70_0(const S70_0 *s) {
    return s->a * 7;
}
static void bump70_0(S70_0 *s, long d) {
    s->a = s->a + d;
}
static long classify70_0(int tag, long a, long b) {
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
static long accum70_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard70_0(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S70_1;

static S70_1 mk70_1(long a) {
    S70_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe70_1(const S70_1 *s) {
    return s->a + s->n0;
}
static long read70_1(const S70_1 *s) {
    return s->a * 4;
}
static void bump70_1(S70_1 *s, long d) {
    s->a = s->a + d;
}
static long classify70_1(int tag, long a, long b) {
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
static long accum70_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard70_1(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S70_2;

static S70_2 mk70_2(long a) {
    S70_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe70_2(const S70_2 *s) {
    return s->a + s->n0;
}
static long read70_2(const S70_2 *s) {
    return s->a * 2;
}
static void bump70_2(S70_2 *s, long d) {
    s->a = s->a + d;
}
static long classify70_2(int tag, long a, long b) {
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
static long accum70_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard70_2(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S70_3;

static S70_3 mk70_3(long a) {
    S70_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe70_3(const S70_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read70_3(const S70_3 *s) {
    return s->a * 6;
}
static void bump70_3(S70_3 *s, long d) {
    s->a = s->a + d;
}
static long classify70_3(int tag, long a, long b) {
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
static long accum70_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard70_3(long x) {
    return x + 3;
}

long f070(long x) {
    long acc = x;
    acc += f041(x + 1);
    S70_0 s0 = mk70_0(acc);
    bump70_0(&s0, 7);
    acc += probe70_0(&s0);
    acc += read70_0(&s0);
    acc += classify70_0(1, acc, acc);
    acc += accum70_0(6);
    acc += guard70_0(acc);
    S70_1 s1 = mk70_1(acc);
    bump70_1(&s1, 5);
    acc += probe70_1(&s1);
    acc += read70_1(&s1);
    acc += classify70_1(1, acc, acc);
    acc += accum70_1(9);
    acc += guard70_1(acc);
    S70_2 s2 = mk70_2(acc);
    bump70_2(&s2, 1);
    acc += probe70_2(&s2);
    acc += read70_2(&s2);
    acc += classify70_2(1, acc, acc);
    acc += accum70_2(6);
    acc += guard70_2(acc);
    S70_3 s3 = mk70_3(acc);
    bump70_3(&s3, 4);
    acc += probe70_3(&s3);
    acc += read70_3(&s3);
    acc += classify70_3(1, acc, acc);
    acc += accum70_3(8);
    acc += guard70_3(acc);
    return clampi(acc);
}
