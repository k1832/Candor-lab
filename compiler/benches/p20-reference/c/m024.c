/* GENERATED C mirror of reference module m024. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S24_0;

static S24_0 mk24_0(long a) {
    S24_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe24_0(const S24_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read24_0(const S24_0 *s) {
    return s->a * 6;
}
static void bump24_0(S24_0 *s, long d) {
    s->a = s->a + d;
}
static long classify24_0(int tag, long a, long b) {
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
static long accum24_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard24_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S24_1;

static S24_1 mk24_1(long a) {
    S24_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe24_1(const S24_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read24_1(const S24_1 *s) {
    return s->a * 3;
}
static void bump24_1(S24_1 *s, long d) {
    s->a = s->a + d;
}
static long classify24_1(int tag, long a, long b) {
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
static long accum24_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard24_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S24_2;

static S24_2 mk24_2(long a) {
    S24_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe24_2(const S24_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read24_2(const S24_2 *s) {
    return s->a * 4;
}
static void bump24_2(S24_2 *s, long d) {
    s->a = s->a + d;
}
static long classify24_2(int tag, long a, long b) {
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
static long accum24_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard24_2(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S24_3;

static S24_3 mk24_3(long a) {
    S24_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe24_3(const S24_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read24_3(const S24_3 *s) {
    return s->a * 7;
}
static void bump24_3(S24_3 *s, long d) {
    s->a = s->a + d;
}
static long classify24_3(int tag, long a, long b) {
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
static long accum24_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard24_3(long x) {
    return x + 6;
}

long f024(long x) {
    long acc = x;
    acc += f000(x + 1);
    acc += f015(x + 2);
    acc += f021(x + 3);
    acc += f023(x + 4);
    S24_0 s0 = mk24_0(acc);
    bump24_0(&s0, 5);
    acc += probe24_0(&s0);
    acc += read24_0(&s0);
    acc += classify24_0(1, acc, acc);
    acc += accum24_0(5);
    acc += guard24_0(acc);
    S24_1 s1 = mk24_1(acc);
    bump24_1(&s1, 7);
    acc += probe24_1(&s1);
    acc += read24_1(&s1);
    acc += classify24_1(1, acc, acc);
    acc += accum24_1(6);
    acc += guard24_1(acc);
    S24_2 s2 = mk24_2(acc);
    bump24_2(&s2, 5);
    acc += probe24_2(&s2);
    acc += read24_2(&s2);
    acc += classify24_2(1, acc, acc);
    acc += accum24_2(8);
    acc += guard24_2(acc);
    S24_3 s3 = mk24_3(acc);
    bump24_3(&s3, 2);
    acc += probe24_3(&s3);
    acc += read24_3(&s3);
    acc += classify24_3(1, acc, acc);
    acc += accum24_3(8);
    acc += guard24_3(acc);
    return clampi(acc);
}
