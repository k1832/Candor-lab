/* GENERATED C mirror of reference module m030. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S30_0;

static S30_0 mk30_0(long a) {
    S30_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe30_0(const S30_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read30_0(const S30_0 *s) {
    return s->a * 4;
}
static void bump30_0(S30_0 *s, long d) {
    s->a = s->a + d;
}
static long classify30_0(int tag, long a, long b) {
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
static long accum30_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard30_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S30_1;

static S30_1 mk30_1(long a) {
    S30_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe30_1(const S30_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read30_1(const S30_1 *s) {
    return s->a * 3;
}
static void bump30_1(S30_1 *s, long d) {
    s->a = s->a + d;
}
static long classify30_1(int tag, long a, long b) {
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
static long accum30_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard30_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S30_2;

static S30_2 mk30_2(long a) {
    S30_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe30_2(const S30_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read30_2(const S30_2 *s) {
    return s->a * 6;
}
static void bump30_2(S30_2 *s, long d) {
    s->a = s->a + d;
}
static long classify30_2(int tag, long a, long b) {
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
static long accum30_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard30_2(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S30_3;

static S30_3 mk30_3(long a) {
    S30_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe30_3(const S30_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read30_3(const S30_3 *s) {
    return s->a * 5;
}
static void bump30_3(S30_3 *s, long d) {
    s->a = s->a + d;
}
static long classify30_3(int tag, long a, long b) {
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
static long accum30_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard30_3(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S30_4;

static S30_4 mk30_4(long a) {
    S30_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe30_4(const S30_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read30_4(const S30_4 *s) {
    return s->a * 2;
}
static void bump30_4(S30_4 *s, long d) {
    s->a = s->a + d;
}
static long classify30_4(int tag, long a, long b) {
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
static long accum30_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard30_4(long x) {
    return x + 7;
}

long f030(long x) {
    long acc = x;
    acc += f003(x + 1);
    S30_0 s0 = mk30_0(acc);
    bump30_0(&s0, 4);
    acc += probe30_0(&s0);
    acc += read30_0(&s0);
    acc += classify30_0(1, acc, acc);
    acc += accum30_0(5);
    acc += guard30_0(acc);
    S30_1 s1 = mk30_1(acc);
    bump30_1(&s1, 9);
    acc += probe30_1(&s1);
    acc += read30_1(&s1);
    acc += classify30_1(1, acc, acc);
    acc += accum30_1(4);
    acc += guard30_1(acc);
    S30_2 s2 = mk30_2(acc);
    bump30_2(&s2, 9);
    acc += probe30_2(&s2);
    acc += read30_2(&s2);
    acc += classify30_2(1, acc, acc);
    acc += accum30_2(5);
    acc += guard30_2(acc);
    S30_3 s3 = mk30_3(acc);
    bump30_3(&s3, 3);
    acc += probe30_3(&s3);
    acc += read30_3(&s3);
    acc += classify30_3(1, acc, acc);
    acc += accum30_3(7);
    acc += guard30_3(acc);
    S30_4 s4 = mk30_4(acc);
    bump30_4(&s4, 7);
    acc += probe30_4(&s4);
    acc += read30_4(&s4);
    acc += classify30_4(1, acc, acc);
    acc += accum30_4(7);
    acc += guard30_4(acc);
    return clampi(acc);
}
