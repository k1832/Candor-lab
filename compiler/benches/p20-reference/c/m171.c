/* GENERATED C mirror of reference module m171. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S171_0;

static S171_0 mk171_0(long a) {
    S171_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe171_0(const S171_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read171_0(const S171_0 *s) {
    return s->a * 3;
}
static void bump171_0(S171_0 *s, long d) {
    s->a = s->a + d;
}
static long classify171_0(int tag, long a, long b) {
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
static long accum171_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard171_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S171_1;

static S171_1 mk171_1(long a) {
    S171_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe171_1(const S171_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read171_1(const S171_1 *s) {
    return s->a * 2;
}
static void bump171_1(S171_1 *s, long d) {
    s->a = s->a + d;
}
static long classify171_1(int tag, long a, long b) {
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
static long accum171_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard171_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S171_2;

static S171_2 mk171_2(long a) {
    S171_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe171_2(const S171_2 *s) {
    return s->a + s->n0;
}
static long read171_2(const S171_2 *s) {
    return s->a * 7;
}
static void bump171_2(S171_2 *s, long d) {
    s->a = s->a + d;
}
static long classify171_2(int tag, long a, long b) {
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
static long accum171_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard171_2(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S171_3;

static S171_3 mk171_3(long a) {
    S171_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe171_3(const S171_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read171_3(const S171_3 *s) {
    return s->a * 7;
}
static void bump171_3(S171_3 *s, long d) {
    s->a = s->a + d;
}
static long classify171_3(int tag, long a, long b) {
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
static long accum171_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard171_3(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S171_4;

static S171_4 mk171_4(long a) {
    S171_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe171_4(const S171_4 *s) {
    return s->a + s->n0;
}
static long read171_4(const S171_4 *s) {
    return s->a * 4;
}
static void bump171_4(S171_4 *s, long d) {
    s->a = s->a + d;
}
static long classify171_4(int tag, long a, long b) {
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
static long accum171_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard171_4(long x) {
    return x + 9;
}

long f171(long x) {
    long acc = x;
    acc += f082(x + 1);
    S171_0 s0 = mk171_0(acc);
    bump171_0(&s0, 2);
    acc += probe171_0(&s0);
    acc += read171_0(&s0);
    acc += classify171_0(1, acc, acc);
    acc += accum171_0(4);
    acc += guard171_0(acc);
    S171_1 s1 = mk171_1(acc);
    bump171_1(&s1, 8);
    acc += probe171_1(&s1);
    acc += read171_1(&s1);
    acc += classify171_1(1, acc, acc);
    acc += accum171_1(8);
    acc += guard171_1(acc);
    S171_2 s2 = mk171_2(acc);
    bump171_2(&s2, 9);
    acc += probe171_2(&s2);
    acc += read171_2(&s2);
    acc += classify171_2(1, acc, acc);
    acc += accum171_2(3);
    acc += guard171_2(acc);
    S171_3 s3 = mk171_3(acc);
    bump171_3(&s3, 3);
    acc += probe171_3(&s3);
    acc += read171_3(&s3);
    acc += classify171_3(1, acc, acc);
    acc += accum171_3(5);
    acc += guard171_3(acc);
    S171_4 s4 = mk171_4(acc);
    bump171_4(&s4, 4);
    acc += probe171_4(&s4);
    acc += read171_4(&s4);
    acc += classify171_4(1, acc, acc);
    acc += accum171_4(3);
    acc += guard171_4(acc);
    return clampi(acc);
}
