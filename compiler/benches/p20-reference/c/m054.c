/* GENERATED C mirror of reference module m054. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S54_0;

static S54_0 mk54_0(long a) {
    S54_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe54_0(const S54_0 *s) {
    return s->a + s->n0;
}
static long read54_0(const S54_0 *s) {
    return s->a * 2;
}
static void bump54_0(S54_0 *s, long d) {
    s->a = s->a + d;
}
static long classify54_0(int tag, long a, long b) {
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
static long accum54_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard54_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S54_1;

static S54_1 mk54_1(long a) {
    S54_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe54_1(const S54_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read54_1(const S54_1 *s) {
    return s->a * 2;
}
static void bump54_1(S54_1 *s, long d) {
    s->a = s->a + d;
}
static long classify54_1(int tag, long a, long b) {
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
static long accum54_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard54_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S54_2;

static S54_2 mk54_2(long a) {
    S54_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe54_2(const S54_2 *s) {
    return s->a + s->n0;
}
static long read54_2(const S54_2 *s) {
    return s->a * 7;
}
static void bump54_2(S54_2 *s, long d) {
    s->a = s->a + d;
}
static long classify54_2(int tag, long a, long b) {
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
static long accum54_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard54_2(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S54_3;

static S54_3 mk54_3(long a) {
    S54_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe54_3(const S54_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read54_3(const S54_3 *s) {
    return s->a * 2;
}
static void bump54_3(S54_3 *s, long d) {
    s->a = s->a + d;
}
static long classify54_3(int tag, long a, long b) {
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
static long accum54_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard54_3(long x) {
    return x + 2;
}

long f054(long x) {
    long acc = x;
    acc += f012(x + 1);
    acc += f027(x + 2);
    S54_0 s0 = mk54_0(acc);
    bump54_0(&s0, 7);
    acc += probe54_0(&s0);
    acc += read54_0(&s0);
    acc += classify54_0(1, acc, acc);
    acc += accum54_0(9);
    acc += guard54_0(acc);
    S54_1 s1 = mk54_1(acc);
    bump54_1(&s1, 7);
    acc += probe54_1(&s1);
    acc += read54_1(&s1);
    acc += classify54_1(1, acc, acc);
    acc += accum54_1(3);
    acc += guard54_1(acc);
    S54_2 s2 = mk54_2(acc);
    bump54_2(&s2, 7);
    acc += probe54_2(&s2);
    acc += read54_2(&s2);
    acc += classify54_2(1, acc, acc);
    acc += accum54_2(3);
    acc += guard54_2(acc);
    S54_3 s3 = mk54_3(acc);
    bump54_3(&s3, 7);
    acc += probe54_3(&s3);
    acc += read54_3(&s3);
    acc += classify54_3(1, acc, acc);
    acc += accum54_3(5);
    acc += guard54_3(acc);
    return clampi(acc);
}
