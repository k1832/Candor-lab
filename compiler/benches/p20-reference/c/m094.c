/* GENERATED C mirror of reference module m094. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S94_0;

static S94_0 mk94_0(long a) {
    S94_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe94_0(const S94_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read94_0(const S94_0 *s) {
    return s->a * 3;
}
static void bump94_0(S94_0 *s, long d) {
    s->a = s->a + d;
}
static long classify94_0(int tag, long a, long b) {
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
static long accum94_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard94_0(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S94_1;

static S94_1 mk94_1(long a) {
    S94_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe94_1(const S94_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read94_1(const S94_1 *s) {
    return s->a * 4;
}
static void bump94_1(S94_1 *s, long d) {
    s->a = s->a + d;
}
static long classify94_1(int tag, long a, long b) {
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
static long accum94_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard94_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S94_2;

static S94_2 mk94_2(long a) {
    S94_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe94_2(const S94_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read94_2(const S94_2 *s) {
    return s->a * 6;
}
static void bump94_2(S94_2 *s, long d) {
    s->a = s->a + d;
}
static long classify94_2(int tag, long a, long b) {
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
static long accum94_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard94_2(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S94_3;

static S94_3 mk94_3(long a) {
    S94_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe94_3(const S94_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read94_3(const S94_3 *s) {
    return s->a * 5;
}
static void bump94_3(S94_3 *s, long d) {
    s->a = s->a + d;
}
static long classify94_3(int tag, long a, long b) {
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
static long accum94_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard94_3(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S94_4;

static S94_4 mk94_4(long a) {
    S94_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe94_4(const S94_4 *s) {
    return s->a + s->n0;
}
static long read94_4(const S94_4 *s) {
    return s->a * 5;
}
static void bump94_4(S94_4 *s, long d) {
    s->a = s->a + d;
}
static long classify94_4(int tag, long a, long b) {
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
static long accum94_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard94_4(long x) {
    return x + 6;
}

long f094(long x) {
    long acc = x;
    acc += f028(x + 1);
    acc += f058(x + 2);
    S94_0 s0 = mk94_0(acc);
    bump94_0(&s0, 1);
    acc += probe94_0(&s0);
    acc += read94_0(&s0);
    acc += classify94_0(1, acc, acc);
    acc += accum94_0(5);
    acc += guard94_0(acc);
    S94_1 s1 = mk94_1(acc);
    bump94_1(&s1, 7);
    acc += probe94_1(&s1);
    acc += read94_1(&s1);
    acc += classify94_1(1, acc, acc);
    acc += accum94_1(4);
    acc += guard94_1(acc);
    S94_2 s2 = mk94_2(acc);
    bump94_2(&s2, 7);
    acc += probe94_2(&s2);
    acc += read94_2(&s2);
    acc += classify94_2(1, acc, acc);
    acc += accum94_2(4);
    acc += guard94_2(acc);
    S94_3 s3 = mk94_3(acc);
    bump94_3(&s3, 1);
    acc += probe94_3(&s3);
    acc += read94_3(&s3);
    acc += classify94_3(1, acc, acc);
    acc += accum94_3(3);
    acc += guard94_3(acc);
    S94_4 s4 = mk94_4(acc);
    bump94_4(&s4, 7);
    acc += probe94_4(&s4);
    acc += read94_4(&s4);
    acc += classify94_4(1, acc, acc);
    acc += accum94_4(4);
    acc += guard94_4(acc);
    return clampi(acc);
}
