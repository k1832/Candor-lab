/* GENERATED C mirror of reference module m043. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S43_0;

static S43_0 mk43_0(long a) {
    S43_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe43_0(const S43_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read43_0(const S43_0 *s) {
    return s->a * 6;
}
static void bump43_0(S43_0 *s, long d) {
    s->a = s->a + d;
}
static long classify43_0(int tag, long a, long b) {
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
static long accum43_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard43_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S43_1;

static S43_1 mk43_1(long a) {
    S43_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe43_1(const S43_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read43_1(const S43_1 *s) {
    return s->a * 4;
}
static void bump43_1(S43_1 *s, long d) {
    s->a = s->a + d;
}
static long classify43_1(int tag, long a, long b) {
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
static long accum43_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard43_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S43_2;

static S43_2 mk43_2(long a) {
    S43_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe43_2(const S43_2 *s) {
    return s->a + s->n0;
}
static long read43_2(const S43_2 *s) {
    return s->a * 4;
}
static void bump43_2(S43_2 *s, long d) {
    s->a = s->a + d;
}
static long classify43_2(int tag, long a, long b) {
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
static long accum43_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard43_2(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S43_3;

static S43_3 mk43_3(long a) {
    S43_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe43_3(const S43_3 *s) {
    return s->a + s->n0;
}
static long read43_3(const S43_3 *s) {
    return s->a * 3;
}
static void bump43_3(S43_3 *s, long d) {
    s->a = s->a + d;
}
static long classify43_3(int tag, long a, long b) {
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
static long accum43_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard43_3(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S43_4;

static S43_4 mk43_4(long a) {
    S43_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe43_4(const S43_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read43_4(const S43_4 *s) {
    return s->a * 2;
}
static void bump43_4(S43_4 *s, long d) {
    s->a = s->a + d;
}
static long classify43_4(int tag, long a, long b) {
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
static long accum43_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard43_4(long x) {
    return x + 1;
}

long f043(long x) {
    long acc = x;
    acc += f003(x + 1);
    acc += f019(x + 2);
    acc += f021(x + 3);
    S43_0 s0 = mk43_0(acc);
    bump43_0(&s0, 3);
    acc += probe43_0(&s0);
    acc += read43_0(&s0);
    acc += classify43_0(1, acc, acc);
    acc += accum43_0(4);
    acc += guard43_0(acc);
    S43_1 s1 = mk43_1(acc);
    bump43_1(&s1, 8);
    acc += probe43_1(&s1);
    acc += read43_1(&s1);
    acc += classify43_1(1, acc, acc);
    acc += accum43_1(6);
    acc += guard43_1(acc);
    S43_2 s2 = mk43_2(acc);
    bump43_2(&s2, 8);
    acc += probe43_2(&s2);
    acc += read43_2(&s2);
    acc += classify43_2(1, acc, acc);
    acc += accum43_2(3);
    acc += guard43_2(acc);
    S43_3 s3 = mk43_3(acc);
    bump43_3(&s3, 9);
    acc += probe43_3(&s3);
    acc += read43_3(&s3);
    acc += classify43_3(1, acc, acc);
    acc += accum43_3(4);
    acc += guard43_3(acc);
    S43_4 s4 = mk43_4(acc);
    bump43_4(&s4, 7);
    acc += probe43_4(&s4);
    acc += read43_4(&s4);
    acc += classify43_4(1, acc, acc);
    acc += accum43_4(8);
    acc += guard43_4(acc);
    return clampi(acc);
}
