/* GENERATED C mirror of reference module m042. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S42_0;

static S42_0 mk42_0(long a) {
    S42_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe42_0(const S42_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read42_0(const S42_0 *s) {
    return s->a * 2;
}
static void bump42_0(S42_0 *s, long d) {
    s->a = s->a + d;
}
static long classify42_0(int tag, long a, long b) {
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
static long accum42_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard42_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S42_1;

static S42_1 mk42_1(long a) {
    S42_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe42_1(const S42_1 *s) {
    return s->a + s->n0;
}
static long read42_1(const S42_1 *s) {
    return s->a * 7;
}
static void bump42_1(S42_1 *s, long d) {
    s->a = s->a + d;
}
static long classify42_1(int tag, long a, long b) {
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
static long accum42_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard42_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S42_2;

static S42_2 mk42_2(long a) {
    S42_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe42_2(const S42_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read42_2(const S42_2 *s) {
    return s->a * 7;
}
static void bump42_2(S42_2 *s, long d) {
    s->a = s->a + d;
}
static long classify42_2(int tag, long a, long b) {
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
static long accum42_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard42_2(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S42_3;

static S42_3 mk42_3(long a) {
    S42_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe42_3(const S42_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read42_3(const S42_3 *s) {
    return s->a * 3;
}
static void bump42_3(S42_3 *s, long d) {
    s->a = s->a + d;
}
static long classify42_3(int tag, long a, long b) {
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
static long accum42_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard42_3(long x) {
    return x + 2;
}

long f042(long x) {
    long acc = x;
    acc += f016(x + 1);
    S42_0 s0 = mk42_0(acc);
    bump42_0(&s0, 1);
    acc += probe42_0(&s0);
    acc += read42_0(&s0);
    acc += classify42_0(1, acc, acc);
    acc += accum42_0(9);
    acc += guard42_0(acc);
    S42_1 s1 = mk42_1(acc);
    bump42_1(&s1, 8);
    acc += probe42_1(&s1);
    acc += read42_1(&s1);
    acc += classify42_1(1, acc, acc);
    acc += accum42_1(8);
    acc += guard42_1(acc);
    S42_2 s2 = mk42_2(acc);
    bump42_2(&s2, 4);
    acc += probe42_2(&s2);
    acc += read42_2(&s2);
    acc += classify42_2(1, acc, acc);
    acc += accum42_2(3);
    acc += guard42_2(acc);
    S42_3 s3 = mk42_3(acc);
    bump42_3(&s3, 3);
    acc += probe42_3(&s3);
    acc += read42_3(&s3);
    acc += classify42_3(1, acc, acc);
    acc += accum42_3(7);
    acc += guard42_3(acc);
    return clampi(acc);
}
