/* GENERATED C mirror of reference module m119. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S119_0;

static S119_0 mk119_0(long a) {
    S119_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe119_0(const S119_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read119_0(const S119_0 *s) {
    return s->a * 6;
}
static void bump119_0(S119_0 *s, long d) {
    s->a = s->a + d;
}
static long classify119_0(int tag, long a, long b) {
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
static long accum119_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard119_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S119_1;

static S119_1 mk119_1(long a) {
    S119_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe119_1(const S119_1 *s) {
    return s->a + s->n0;
}
static long read119_1(const S119_1 *s) {
    return s->a * 5;
}
static void bump119_1(S119_1 *s, long d) {
    s->a = s->a + d;
}
static long classify119_1(int tag, long a, long b) {
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
static long accum119_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard119_1(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S119_2;

static S119_2 mk119_2(long a) {
    S119_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe119_2(const S119_2 *s) {
    return s->a + s->n0;
}
static long read119_2(const S119_2 *s) {
    return s->a * 2;
}
static void bump119_2(S119_2 *s, long d) {
    s->a = s->a + d;
}
static long classify119_2(int tag, long a, long b) {
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
static long accum119_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard119_2(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S119_3;

static S119_3 mk119_3(long a) {
    S119_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe119_3(const S119_3 *s) {
    return s->a + s->n0;
}
static long read119_3(const S119_3 *s) {
    return s->a * 4;
}
static void bump119_3(S119_3 *s, long d) {
    s->a = s->a + d;
}
static long classify119_3(int tag, long a, long b) {
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
static long accum119_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard119_3(long x) {
    return x + 2;
}

long f119(long x) {
    long acc = x;
    acc += f031(x + 1);
    acc += f070(x + 2);
    acc += f099(x + 3);
    S119_0 s0 = mk119_0(acc);
    bump119_0(&s0, 3);
    acc += probe119_0(&s0);
    acc += read119_0(&s0);
    acc += classify119_0(1, acc, acc);
    acc += accum119_0(3);
    acc += guard119_0(acc);
    S119_1 s1 = mk119_1(acc);
    bump119_1(&s1, 9);
    acc += probe119_1(&s1);
    acc += read119_1(&s1);
    acc += classify119_1(1, acc, acc);
    acc += accum119_1(7);
    acc += guard119_1(acc);
    S119_2 s2 = mk119_2(acc);
    bump119_2(&s2, 1);
    acc += probe119_2(&s2);
    acc += read119_2(&s2);
    acc += classify119_2(1, acc, acc);
    acc += accum119_2(5);
    acc += guard119_2(acc);
    S119_3 s3 = mk119_3(acc);
    bump119_3(&s3, 2);
    acc += probe119_3(&s3);
    acc += read119_3(&s3);
    acc += classify119_3(1, acc, acc);
    acc += accum119_3(7);
    acc += guard119_3(acc);
    return clampi(acc);
}
