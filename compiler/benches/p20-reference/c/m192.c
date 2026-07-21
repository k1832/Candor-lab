/* GENERATED C mirror of reference module m192. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S192_0;

static S192_0 mk192_0(long a) {
    S192_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe192_0(const S192_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read192_0(const S192_0 *s) {
    return s->a * 4;
}
static void bump192_0(S192_0 *s, long d) {
    s->a = s->a + d;
}
static long classify192_0(int tag, long a, long b) {
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
static long accum192_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard192_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S192_1;

static S192_1 mk192_1(long a) {
    S192_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe192_1(const S192_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read192_1(const S192_1 *s) {
    return s->a * 7;
}
static void bump192_1(S192_1 *s, long d) {
    s->a = s->a + d;
}
static long classify192_1(int tag, long a, long b) {
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
static long accum192_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard192_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S192_2;

static S192_2 mk192_2(long a) {
    S192_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe192_2(const S192_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read192_2(const S192_2 *s) {
    return s->a * 3;
}
static void bump192_2(S192_2 *s, long d) {
    s->a = s->a + d;
}
static long classify192_2(int tag, long a, long b) {
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
static long accum192_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard192_2(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S192_3;

static S192_3 mk192_3(long a) {
    S192_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe192_3(const S192_3 *s) {
    return s->a + s->n0;
}
static long read192_3(const S192_3 *s) {
    return s->a * 2;
}
static void bump192_3(S192_3 *s, long d) {
    s->a = s->a + d;
}
static long classify192_3(int tag, long a, long b) {
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
static long accum192_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard192_3(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S192_4;

static S192_4 mk192_4(long a) {
    S192_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe192_4(const S192_4 *s) {
    return s->a + s->n0;
}
static long read192_4(const S192_4 *s) {
    return s->a * 6;
}
static void bump192_4(S192_4 *s, long d) {
    s->a = s->a + d;
}
static long classify192_4(int tag, long a, long b) {
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
static long accum192_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard192_4(long x) {
    return x + 4;
}

long f192(long x) {
    long acc = x;
    acc += f029(x + 1);
    S192_0 s0 = mk192_0(acc);
    bump192_0(&s0, 4);
    acc += probe192_0(&s0);
    acc += read192_0(&s0);
    acc += classify192_0(1, acc, acc);
    acc += accum192_0(4);
    acc += guard192_0(acc);
    S192_1 s1 = mk192_1(acc);
    bump192_1(&s1, 4);
    acc += probe192_1(&s1);
    acc += read192_1(&s1);
    acc += classify192_1(1, acc, acc);
    acc += accum192_1(6);
    acc += guard192_1(acc);
    S192_2 s2 = mk192_2(acc);
    bump192_2(&s2, 2);
    acc += probe192_2(&s2);
    acc += read192_2(&s2);
    acc += classify192_2(1, acc, acc);
    acc += accum192_2(3);
    acc += guard192_2(acc);
    S192_3 s3 = mk192_3(acc);
    bump192_3(&s3, 9);
    acc += probe192_3(&s3);
    acc += read192_3(&s3);
    acc += classify192_3(1, acc, acc);
    acc += accum192_3(3);
    acc += guard192_3(acc);
    S192_4 s4 = mk192_4(acc);
    bump192_4(&s4, 7);
    acc += probe192_4(&s4);
    acc += read192_4(&s4);
    acc += classify192_4(1, acc, acc);
    acc += accum192_4(9);
    acc += guard192_4(acc);
    return clampi(acc);
}
