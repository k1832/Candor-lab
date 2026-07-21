/* GENERATED C mirror of reference module m123. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S123_0;

static S123_0 mk123_0(long a) {
    S123_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe123_0(const S123_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read123_0(const S123_0 *s) {
    return s->a * 7;
}
static void bump123_0(S123_0 *s, long d) {
    s->a = s->a + d;
}
static long classify123_0(int tag, long a, long b) {
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
static long accum123_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard123_0(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S123_1;

static S123_1 mk123_1(long a) {
    S123_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe123_1(const S123_1 *s) {
    return s->a + s->n0;
}
static long read123_1(const S123_1 *s) {
    return s->a * 7;
}
static void bump123_1(S123_1 *s, long d) {
    s->a = s->a + d;
}
static long classify123_1(int tag, long a, long b) {
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
static long accum123_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard123_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S123_2;

static S123_2 mk123_2(long a) {
    S123_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe123_2(const S123_2 *s) {
    return s->a + s->n0;
}
static long read123_2(const S123_2 *s) {
    return s->a * 3;
}
static void bump123_2(S123_2 *s, long d) {
    s->a = s->a + d;
}
static long classify123_2(int tag, long a, long b) {
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
static long accum123_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard123_2(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S123_3;

static S123_3 mk123_3(long a) {
    S123_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe123_3(const S123_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read123_3(const S123_3 *s) {
    return s->a * 3;
}
static void bump123_3(S123_3 *s, long d) {
    s->a = s->a + d;
}
static long classify123_3(int tag, long a, long b) {
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
static long accum123_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard123_3(long x) {
    return x + 9;
}

long f123(long x) {
    long acc = x;
    acc += f015(x + 1);
    S123_0 s0 = mk123_0(acc);
    bump123_0(&s0, 7);
    acc += probe123_0(&s0);
    acc += read123_0(&s0);
    acc += classify123_0(1, acc, acc);
    acc += accum123_0(7);
    acc += guard123_0(acc);
    S123_1 s1 = mk123_1(acc);
    bump123_1(&s1, 6);
    acc += probe123_1(&s1);
    acc += read123_1(&s1);
    acc += classify123_1(1, acc, acc);
    acc += accum123_1(8);
    acc += guard123_1(acc);
    S123_2 s2 = mk123_2(acc);
    bump123_2(&s2, 6);
    acc += probe123_2(&s2);
    acc += read123_2(&s2);
    acc += classify123_2(1, acc, acc);
    acc += accum123_2(5);
    acc += guard123_2(acc);
    S123_3 s3 = mk123_3(acc);
    bump123_3(&s3, 2);
    acc += probe123_3(&s3);
    acc += read123_3(&s3);
    acc += classify123_3(1, acc, acc);
    acc += accum123_3(4);
    acc += guard123_3(acc);
    return clampi(acc);
}
