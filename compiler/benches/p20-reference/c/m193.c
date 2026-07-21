/* GENERATED C mirror of reference module m193. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S193_0;

static S193_0 mk193_0(long a) {
    S193_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe193_0(const S193_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read193_0(const S193_0 *s) {
    return s->a * 5;
}
static void bump193_0(S193_0 *s, long d) {
    s->a = s->a + d;
}
static long classify193_0(int tag, long a, long b) {
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
static long accum193_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard193_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S193_1;

static S193_1 mk193_1(long a) {
    S193_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe193_1(const S193_1 *s) {
    return s->a + s->n0;
}
static long read193_1(const S193_1 *s) {
    return s->a * 3;
}
static void bump193_1(S193_1 *s, long d) {
    s->a = s->a + d;
}
static long classify193_1(int tag, long a, long b) {
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
static long accum193_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard193_1(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S193_2;

static S193_2 mk193_2(long a) {
    S193_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe193_2(const S193_2 *s) {
    return s->a + s->n0;
}
static long read193_2(const S193_2 *s) {
    return s->a * 4;
}
static void bump193_2(S193_2 *s, long d) {
    s->a = s->a + d;
}
static long classify193_2(int tag, long a, long b) {
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
static long accum193_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard193_2(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S193_3;

static S193_3 mk193_3(long a) {
    S193_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe193_3(const S193_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read193_3(const S193_3 *s) {
    return s->a * 4;
}
static void bump193_3(S193_3 *s, long d) {
    s->a = s->a + d;
}
static long classify193_3(int tag, long a, long b) {
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
static long accum193_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard193_3(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S193_4;

static S193_4 mk193_4(long a) {
    S193_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe193_4(const S193_4 *s) {
    return s->a + s->n0;
}
static long read193_4(const S193_4 *s) {
    return s->a * 2;
}
static void bump193_4(S193_4 *s, long d) {
    s->a = s->a + d;
}
static long classify193_4(int tag, long a, long b) {
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
static long accum193_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard193_4(long x) {
    return x + 2;
}

long f193(long x) {
    long acc = x;
    acc += f134(x + 1);
    acc += f151(x + 2);
    S193_0 s0 = mk193_0(acc);
    bump193_0(&s0, 3);
    acc += probe193_0(&s0);
    acc += read193_0(&s0);
    acc += classify193_0(1, acc, acc);
    acc += accum193_0(5);
    acc += guard193_0(acc);
    S193_1 s1 = mk193_1(acc);
    bump193_1(&s1, 4);
    acc += probe193_1(&s1);
    acc += read193_1(&s1);
    acc += classify193_1(1, acc, acc);
    acc += accum193_1(3);
    acc += guard193_1(acc);
    S193_2 s2 = mk193_2(acc);
    bump193_2(&s2, 5);
    acc += probe193_2(&s2);
    acc += read193_2(&s2);
    acc += classify193_2(1, acc, acc);
    acc += accum193_2(4);
    acc += guard193_2(acc);
    S193_3 s3 = mk193_3(acc);
    bump193_3(&s3, 5);
    acc += probe193_3(&s3);
    acc += read193_3(&s3);
    acc += classify193_3(1, acc, acc);
    acc += accum193_3(4);
    acc += guard193_3(acc);
    S193_4 s4 = mk193_4(acc);
    bump193_4(&s4, 3);
    acc += probe193_4(&s4);
    acc += read193_4(&s4);
    acc += classify193_4(1, acc, acc);
    acc += accum193_4(3);
    acc += guard193_4(acc);
    return clampi(acc);
}
