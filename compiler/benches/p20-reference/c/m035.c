/* GENERATED C mirror of reference module m035. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S35_0;

static S35_0 mk35_0(long a) {
    S35_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe35_0(const S35_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read35_0(const S35_0 *s) {
    return s->a * 4;
}
static void bump35_0(S35_0 *s, long d) {
    s->a = s->a + d;
}
static long classify35_0(int tag, long a, long b) {
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
static long accum35_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard35_0(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S35_1;

static S35_1 mk35_1(long a) {
    S35_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe35_1(const S35_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read35_1(const S35_1 *s) {
    return s->a * 5;
}
static void bump35_1(S35_1 *s, long d) {
    s->a = s->a + d;
}
static long classify35_1(int tag, long a, long b) {
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
static long accum35_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard35_1(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S35_2;

static S35_2 mk35_2(long a) {
    S35_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe35_2(const S35_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read35_2(const S35_2 *s) {
    return s->a * 4;
}
static void bump35_2(S35_2 *s, long d) {
    s->a = s->a + d;
}
static long classify35_2(int tag, long a, long b) {
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
static long accum35_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard35_2(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S35_3;

static S35_3 mk35_3(long a) {
    S35_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe35_3(const S35_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read35_3(const S35_3 *s) {
    return s->a * 7;
}
static void bump35_3(S35_3 *s, long d) {
    s->a = s->a + d;
}
static long classify35_3(int tag, long a, long b) {
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
static long accum35_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard35_3(long x) {
    return x + 6;
}

long f035(long x) {
    long acc = x;
    acc += f002(x + 1);
    acc += f008(x + 2);
    acc += f014(x + 3);
    S35_0 s0 = mk35_0(acc);
    bump35_0(&s0, 3);
    acc += probe35_0(&s0);
    acc += read35_0(&s0);
    acc += classify35_0(1, acc, acc);
    acc += accum35_0(8);
    acc += guard35_0(acc);
    S35_1 s1 = mk35_1(acc);
    bump35_1(&s1, 9);
    acc += probe35_1(&s1);
    acc += read35_1(&s1);
    acc += classify35_1(1, acc, acc);
    acc += accum35_1(7);
    acc += guard35_1(acc);
    S35_2 s2 = mk35_2(acc);
    bump35_2(&s2, 6);
    acc += probe35_2(&s2);
    acc += read35_2(&s2);
    acc += classify35_2(1, acc, acc);
    acc += accum35_2(6);
    acc += guard35_2(acc);
    S35_3 s3 = mk35_3(acc);
    bump35_3(&s3, 5);
    acc += probe35_3(&s3);
    acc += read35_3(&s3);
    acc += classify35_3(1, acc, acc);
    acc += accum35_3(4);
    acc += guard35_3(acc);
    return clampi(acc);
}
