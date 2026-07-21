/* GENERATED C mirror of reference module m021. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S21_0;

static S21_0 mk21_0(long a) {
    S21_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe21_0(const S21_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read21_0(const S21_0 *s) {
    return s->a * 7;
}
static void bump21_0(S21_0 *s, long d) {
    s->a = s->a + d;
}
static long classify21_0(int tag, long a, long b) {
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
static long accum21_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard21_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S21_1;

static S21_1 mk21_1(long a) {
    S21_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe21_1(const S21_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read21_1(const S21_1 *s) {
    return s->a * 2;
}
static void bump21_1(S21_1 *s, long d) {
    s->a = s->a + d;
}
static long classify21_1(int tag, long a, long b) {
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
static long accum21_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard21_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S21_2;

static S21_2 mk21_2(long a) {
    S21_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe21_2(const S21_2 *s) {
    return s->a + s->n0;
}
static long read21_2(const S21_2 *s) {
    return s->a * 6;
}
static void bump21_2(S21_2 *s, long d) {
    s->a = s->a + d;
}
static long classify21_2(int tag, long a, long b) {
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
static long accum21_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard21_2(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S21_3;

static S21_3 mk21_3(long a) {
    S21_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe21_3(const S21_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read21_3(const S21_3 *s) {
    return s->a * 7;
}
static void bump21_3(S21_3 *s, long d) {
    s->a = s->a + d;
}
static long classify21_3(int tag, long a, long b) {
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
static long accum21_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard21_3(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S21_4;

static S21_4 mk21_4(long a) {
    S21_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe21_4(const S21_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read21_4(const S21_4 *s) {
    return s->a * 7;
}
static void bump21_4(S21_4 *s, long d) {
    s->a = s->a + d;
}
static long classify21_4(int tag, long a, long b) {
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
static long accum21_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard21_4(long x) {
    return x + 9;
}

long f021(long x) {
    long acc = x;
    acc += f003(x + 1);
    S21_0 s0 = mk21_0(acc);
    bump21_0(&s0, 1);
    acc += probe21_0(&s0);
    acc += read21_0(&s0);
    acc += classify21_0(1, acc, acc);
    acc += accum21_0(3);
    acc += guard21_0(acc);
    S21_1 s1 = mk21_1(acc);
    bump21_1(&s1, 4);
    acc += probe21_1(&s1);
    acc += read21_1(&s1);
    acc += classify21_1(1, acc, acc);
    acc += accum21_1(6);
    acc += guard21_1(acc);
    S21_2 s2 = mk21_2(acc);
    bump21_2(&s2, 8);
    acc += probe21_2(&s2);
    acc += read21_2(&s2);
    acc += classify21_2(1, acc, acc);
    acc += accum21_2(4);
    acc += guard21_2(acc);
    S21_3 s3 = mk21_3(acc);
    bump21_3(&s3, 4);
    acc += probe21_3(&s3);
    acc += read21_3(&s3);
    acc += classify21_3(1, acc, acc);
    acc += accum21_3(4);
    acc += guard21_3(acc);
    S21_4 s4 = mk21_4(acc);
    bump21_4(&s4, 9);
    acc += probe21_4(&s4);
    acc += read21_4(&s4);
    acc += classify21_4(1, acc, acc);
    acc += accum21_4(7);
    acc += guard21_4(acc);
    return clampi(acc);
}
