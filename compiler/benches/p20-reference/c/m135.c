/* GENERATED C mirror of reference module m135. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S135_0;

static S135_0 mk135_0(long a) {
    S135_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe135_0(const S135_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read135_0(const S135_0 *s) {
    return s->a * 7;
}
static void bump135_0(S135_0 *s, long d) {
    s->a = s->a + d;
}
static long classify135_0(int tag, long a, long b) {
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
static long accum135_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard135_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S135_1;

static S135_1 mk135_1(long a) {
    S135_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe135_1(const S135_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read135_1(const S135_1 *s) {
    return s->a * 7;
}
static void bump135_1(S135_1 *s, long d) {
    s->a = s->a + d;
}
static long classify135_1(int tag, long a, long b) {
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
static long accum135_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard135_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S135_2;

static S135_2 mk135_2(long a) {
    S135_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe135_2(const S135_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read135_2(const S135_2 *s) {
    return s->a * 4;
}
static void bump135_2(S135_2 *s, long d) {
    s->a = s->a + d;
}
static long classify135_2(int tag, long a, long b) {
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
static long accum135_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard135_2(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S135_3;

static S135_3 mk135_3(long a) {
    S135_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe135_3(const S135_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read135_3(const S135_3 *s) {
    return s->a * 6;
}
static void bump135_3(S135_3 *s, long d) {
    s->a = s->a + d;
}
static long classify135_3(int tag, long a, long b) {
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
static long accum135_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard135_3(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S135_4;

static S135_4 mk135_4(long a) {
    S135_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe135_4(const S135_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read135_4(const S135_4 *s) {
    return s->a * 4;
}
static void bump135_4(S135_4 *s, long d) {
    s->a = s->a + d;
}
static long classify135_4(int tag, long a, long b) {
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
static long accum135_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard135_4(long x) {
    return x + 2;
}

long f135(long x) {
    long acc = x;
    acc += f054(x + 1);
    acc += f070(x + 2);
    S135_0 s0 = mk135_0(acc);
    bump135_0(&s0, 5);
    acc += probe135_0(&s0);
    acc += read135_0(&s0);
    acc += classify135_0(1, acc, acc);
    acc += accum135_0(7);
    acc += guard135_0(acc);
    S135_1 s1 = mk135_1(acc);
    bump135_1(&s1, 2);
    acc += probe135_1(&s1);
    acc += read135_1(&s1);
    acc += classify135_1(1, acc, acc);
    acc += accum135_1(4);
    acc += guard135_1(acc);
    S135_2 s2 = mk135_2(acc);
    bump135_2(&s2, 2);
    acc += probe135_2(&s2);
    acc += read135_2(&s2);
    acc += classify135_2(1, acc, acc);
    acc += accum135_2(6);
    acc += guard135_2(acc);
    S135_3 s3 = mk135_3(acc);
    bump135_3(&s3, 1);
    acc += probe135_3(&s3);
    acc += read135_3(&s3);
    acc += classify135_3(1, acc, acc);
    acc += accum135_3(6);
    acc += guard135_3(acc);
    S135_4 s4 = mk135_4(acc);
    bump135_4(&s4, 1);
    acc += probe135_4(&s4);
    acc += read135_4(&s4);
    acc += classify135_4(1, acc, acc);
    acc += accum135_4(8);
    acc += guard135_4(acc);
    return clampi(acc);
}
