/* GENERATED C mirror of reference module m003. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S3_0;

static S3_0 mk3_0(long a) {
    S3_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe3_0(const S3_0 *s) {
    return s->a + s->n0;
}
static long read3_0(const S3_0 *s) {
    return s->a * 7;
}
static void bump3_0(S3_0 *s, long d) {
    s->a = s->a + d;
}
static long classify3_0(int tag, long a, long b) {
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
static long accum3_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard3_0(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S3_1;

static S3_1 mk3_1(long a) {
    S3_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe3_1(const S3_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read3_1(const S3_1 *s) {
    return s->a * 2;
}
static void bump3_1(S3_1 *s, long d) {
    s->a = s->a + d;
}
static long classify3_1(int tag, long a, long b) {
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
static long accum3_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard3_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S3_2;

static S3_2 mk3_2(long a) {
    S3_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe3_2(const S3_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read3_2(const S3_2 *s) {
    return s->a * 6;
}
static void bump3_2(S3_2 *s, long d) {
    s->a = s->a + d;
}
static long classify3_2(int tag, long a, long b) {
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
static long accum3_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard3_2(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S3_3;

static S3_3 mk3_3(long a) {
    S3_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe3_3(const S3_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read3_3(const S3_3 *s) {
    return s->a * 6;
}
static void bump3_3(S3_3 *s, long d) {
    s->a = s->a + d;
}
static long classify3_3(int tag, long a, long b) {
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
static long accum3_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard3_3(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S3_4;

static S3_4 mk3_4(long a) {
    S3_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe3_4(const S3_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read3_4(const S3_4 *s) {
    return s->a * 6;
}
static void bump3_4(S3_4 *s, long d) {
    s->a = s->a + d;
}
static long classify3_4(int tag, long a, long b) {
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
static long accum3_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard3_4(long x) {
    return x + 2;
}

long f003(long x) {
    long acc = x;
    S3_0 s0 = mk3_0(acc);
    bump3_0(&s0, 2);
    acc += probe3_0(&s0);
    acc += read3_0(&s0);
    acc += classify3_0(1, acc, acc);
    acc += accum3_0(4);
    acc += guard3_0(acc);
    S3_1 s1 = mk3_1(acc);
    bump3_1(&s1, 3);
    acc += probe3_1(&s1);
    acc += read3_1(&s1);
    acc += classify3_1(1, acc, acc);
    acc += accum3_1(3);
    acc += guard3_1(acc);
    S3_2 s2 = mk3_2(acc);
    bump3_2(&s2, 3);
    acc += probe3_2(&s2);
    acc += read3_2(&s2);
    acc += classify3_2(1, acc, acc);
    acc += accum3_2(3);
    acc += guard3_2(acc);
    S3_3 s3 = mk3_3(acc);
    bump3_3(&s3, 9);
    acc += probe3_3(&s3);
    acc += read3_3(&s3);
    acc += classify3_3(1, acc, acc);
    acc += accum3_3(3);
    acc += guard3_3(acc);
    S3_4 s4 = mk3_4(acc);
    bump3_4(&s4, 7);
    acc += probe3_4(&s4);
    acc += read3_4(&s4);
    acc += classify3_4(1, acc, acc);
    acc += accum3_4(5);
    acc += guard3_4(acc);
    return clampi(acc);
}
