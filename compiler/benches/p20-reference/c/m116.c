/* GENERATED C mirror of reference module m116. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S116_0;

static S116_0 mk116_0(long a) {
    S116_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe116_0(const S116_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read116_0(const S116_0 *s) {
    return s->a * 7;
}
static void bump116_0(S116_0 *s, long d) {
    s->a = s->a + d;
}
static long classify116_0(int tag, long a, long b) {
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
static long accum116_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard116_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S116_1;

static S116_1 mk116_1(long a) {
    S116_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe116_1(const S116_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read116_1(const S116_1 *s) {
    return s->a * 3;
}
static void bump116_1(S116_1 *s, long d) {
    s->a = s->a + d;
}
static long classify116_1(int tag, long a, long b) {
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
static long accum116_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard116_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S116_2;

static S116_2 mk116_2(long a) {
    S116_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe116_2(const S116_2 *s) {
    return s->a + s->n0;
}
static long read116_2(const S116_2 *s) {
    return s->a * 5;
}
static void bump116_2(S116_2 *s, long d) {
    s->a = s->a + d;
}
static long classify116_2(int tag, long a, long b) {
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
static long accum116_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard116_2(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S116_3;

static S116_3 mk116_3(long a) {
    S116_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe116_3(const S116_3 *s) {
    return s->a + s->n0;
}
static long read116_3(const S116_3 *s) {
    return s->a * 5;
}
static void bump116_3(S116_3 *s, long d) {
    s->a = s->a + d;
}
static long classify116_3(int tag, long a, long b) {
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
static long accum116_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard116_3(long x) {
    return x + 2;
}

long f116(long x) {
    long acc = x;
    acc += f006(x + 1);
    acc += f028(x + 2);
    acc += f039(x + 3);
    acc += f103(x + 4);
    S116_0 s0 = mk116_0(acc);
    bump116_0(&s0, 9);
    acc += probe116_0(&s0);
    acc += read116_0(&s0);
    acc += classify116_0(1, acc, acc);
    acc += accum116_0(9);
    acc += guard116_0(acc);
    S116_1 s1 = mk116_1(acc);
    bump116_1(&s1, 9);
    acc += probe116_1(&s1);
    acc += read116_1(&s1);
    acc += classify116_1(1, acc, acc);
    acc += accum116_1(9);
    acc += guard116_1(acc);
    S116_2 s2 = mk116_2(acc);
    bump116_2(&s2, 1);
    acc += probe116_2(&s2);
    acc += read116_2(&s2);
    acc += classify116_2(1, acc, acc);
    acc += accum116_2(4);
    acc += guard116_2(acc);
    S116_3 s3 = mk116_3(acc);
    bump116_3(&s3, 9);
    acc += probe116_3(&s3);
    acc += read116_3(&s3);
    acc += classify116_3(1, acc, acc);
    acc += accum116_3(6);
    acc += guard116_3(acc);
    return clampi(acc);
}
