/* GENERATED C mirror of reference module m129. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S129_0;

static S129_0 mk129_0(long a) {
    S129_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe129_0(const S129_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read129_0(const S129_0 *s) {
    return s->a * 5;
}
static void bump129_0(S129_0 *s, long d) {
    s->a = s->a + d;
}
static long classify129_0(int tag, long a, long b) {
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
static long accum129_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard129_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S129_1;

static S129_1 mk129_1(long a) {
    S129_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe129_1(const S129_1 *s) {
    return s->a + s->n0;
}
static long read129_1(const S129_1 *s) {
    return s->a * 2;
}
static void bump129_1(S129_1 *s, long d) {
    s->a = s->a + d;
}
static long classify129_1(int tag, long a, long b) {
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
static long accum129_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard129_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S129_2;

static S129_2 mk129_2(long a) {
    S129_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe129_2(const S129_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read129_2(const S129_2 *s) {
    return s->a * 7;
}
static void bump129_2(S129_2 *s, long d) {
    s->a = s->a + d;
}
static long classify129_2(int tag, long a, long b) {
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
static long accum129_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard129_2(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S129_3;

static S129_3 mk129_3(long a) {
    S129_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe129_3(const S129_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read129_3(const S129_3 *s) {
    return s->a * 3;
}
static void bump129_3(S129_3 *s, long d) {
    s->a = s->a + d;
}
static long classify129_3(int tag, long a, long b) {
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
static long accum129_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard129_3(long x) {
    return x + 4;
}

long f129(long x) {
    long acc = x;
    acc += f088(x + 1);
    S129_0 s0 = mk129_0(acc);
    bump129_0(&s0, 5);
    acc += probe129_0(&s0);
    acc += read129_0(&s0);
    acc += classify129_0(1, acc, acc);
    acc += accum129_0(3);
    acc += guard129_0(acc);
    S129_1 s1 = mk129_1(acc);
    bump129_1(&s1, 4);
    acc += probe129_1(&s1);
    acc += read129_1(&s1);
    acc += classify129_1(1, acc, acc);
    acc += accum129_1(5);
    acc += guard129_1(acc);
    S129_2 s2 = mk129_2(acc);
    bump129_2(&s2, 9);
    acc += probe129_2(&s2);
    acc += read129_2(&s2);
    acc += classify129_2(1, acc, acc);
    acc += accum129_2(3);
    acc += guard129_2(acc);
    S129_3 s3 = mk129_3(acc);
    bump129_3(&s3, 2);
    acc += probe129_3(&s3);
    acc += read129_3(&s3);
    acc += classify129_3(1, acc, acc);
    acc += accum129_3(7);
    acc += guard129_3(acc);
    return clampi(acc);
}
