/* GENERATED C mirror of reference module m168. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S168_0;

static S168_0 mk168_0(long a) {
    S168_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe168_0(const S168_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read168_0(const S168_0 *s) {
    return s->a * 2;
}
static void bump168_0(S168_0 *s, long d) {
    s->a = s->a + d;
}
static long classify168_0(int tag, long a, long b) {
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
static long accum168_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard168_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S168_1;

static S168_1 mk168_1(long a) {
    S168_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe168_1(const S168_1 *s) {
    return s->a + s->n0;
}
static long read168_1(const S168_1 *s) {
    return s->a * 5;
}
static void bump168_1(S168_1 *s, long d) {
    s->a = s->a + d;
}
static long classify168_1(int tag, long a, long b) {
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
static long accum168_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard168_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S168_2;

static S168_2 mk168_2(long a) {
    S168_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe168_2(const S168_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read168_2(const S168_2 *s) {
    return s->a * 2;
}
static void bump168_2(S168_2 *s, long d) {
    s->a = s->a + d;
}
static long classify168_2(int tag, long a, long b) {
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
static long accum168_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard168_2(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S168_3;

static S168_3 mk168_3(long a) {
    S168_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe168_3(const S168_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read168_3(const S168_3 *s) {
    return s->a * 5;
}
static void bump168_3(S168_3 *s, long d) {
    s->a = s->a + d;
}
static long classify168_3(int tag, long a, long b) {
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
static long accum168_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard168_3(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S168_4;

static S168_4 mk168_4(long a) {
    S168_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe168_4(const S168_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read168_4(const S168_4 *s) {
    return s->a * 7;
}
static void bump168_4(S168_4 *s, long d) {
    s->a = s->a + d;
}
static long classify168_4(int tag, long a, long b) {
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
static long accum168_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard168_4(long x) {
    return x + 9;
}

long f168(long x) {
    long acc = x;
    acc += f068(x + 1);
    acc += f074(x + 2);
    acc += f118(x + 3);
    S168_0 s0 = mk168_0(acc);
    bump168_0(&s0, 9);
    acc += probe168_0(&s0);
    acc += read168_0(&s0);
    acc += classify168_0(1, acc, acc);
    acc += accum168_0(6);
    acc += guard168_0(acc);
    S168_1 s1 = mk168_1(acc);
    bump168_1(&s1, 6);
    acc += probe168_1(&s1);
    acc += read168_1(&s1);
    acc += classify168_1(1, acc, acc);
    acc += accum168_1(9);
    acc += guard168_1(acc);
    S168_2 s2 = mk168_2(acc);
    bump168_2(&s2, 9);
    acc += probe168_2(&s2);
    acc += read168_2(&s2);
    acc += classify168_2(1, acc, acc);
    acc += accum168_2(4);
    acc += guard168_2(acc);
    S168_3 s3 = mk168_3(acc);
    bump168_3(&s3, 5);
    acc += probe168_3(&s3);
    acc += read168_3(&s3);
    acc += classify168_3(1, acc, acc);
    acc += accum168_3(3);
    acc += guard168_3(acc);
    S168_4 s4 = mk168_4(acc);
    bump168_4(&s4, 1);
    acc += probe168_4(&s4);
    acc += read168_4(&s4);
    acc += classify168_4(1, acc, acc);
    acc += accum168_4(8);
    acc += guard168_4(acc);
    return clampi(acc);
}
