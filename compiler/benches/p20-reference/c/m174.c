/* GENERATED C mirror of reference module m174. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S174_0;

static S174_0 mk174_0(long a) {
    S174_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe174_0(const S174_0 *s) {
    return s->a + s->n0;
}
static long read174_0(const S174_0 *s) {
    return s->a * 3;
}
static void bump174_0(S174_0 *s, long d) {
    s->a = s->a + d;
}
static long classify174_0(int tag, long a, long b) {
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
static long accum174_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard174_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S174_1;

static S174_1 mk174_1(long a) {
    S174_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe174_1(const S174_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read174_1(const S174_1 *s) {
    return s->a * 7;
}
static void bump174_1(S174_1 *s, long d) {
    s->a = s->a + d;
}
static long classify174_1(int tag, long a, long b) {
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
static long accum174_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard174_1(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S174_2;

static S174_2 mk174_2(long a) {
    S174_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe174_2(const S174_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read174_2(const S174_2 *s) {
    return s->a * 6;
}
static void bump174_2(S174_2 *s, long d) {
    s->a = s->a + d;
}
static long classify174_2(int tag, long a, long b) {
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
static long accum174_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard174_2(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S174_3;

static S174_3 mk174_3(long a) {
    S174_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe174_3(const S174_3 *s) {
    return s->a + s->n0;
}
static long read174_3(const S174_3 *s) {
    return s->a * 4;
}
static void bump174_3(S174_3 *s, long d) {
    s->a = s->a + d;
}
static long classify174_3(int tag, long a, long b) {
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
static long accum174_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard174_3(long x) {
    return x + 8;
}

long f174(long x) {
    long acc = x;
    acc += f045(x + 1);
    S174_0 s0 = mk174_0(acc);
    bump174_0(&s0, 9);
    acc += probe174_0(&s0);
    acc += read174_0(&s0);
    acc += classify174_0(1, acc, acc);
    acc += accum174_0(5);
    acc += guard174_0(acc);
    S174_1 s1 = mk174_1(acc);
    bump174_1(&s1, 9);
    acc += probe174_1(&s1);
    acc += read174_1(&s1);
    acc += classify174_1(1, acc, acc);
    acc += accum174_1(8);
    acc += guard174_1(acc);
    S174_2 s2 = mk174_2(acc);
    bump174_2(&s2, 1);
    acc += probe174_2(&s2);
    acc += read174_2(&s2);
    acc += classify174_2(1, acc, acc);
    acc += accum174_2(3);
    acc += guard174_2(acc);
    S174_3 s3 = mk174_3(acc);
    bump174_3(&s3, 4);
    acc += probe174_3(&s3);
    acc += read174_3(&s3);
    acc += classify174_3(1, acc, acc);
    acc += accum174_3(6);
    acc += guard174_3(acc);
    return clampi(acc);
}
