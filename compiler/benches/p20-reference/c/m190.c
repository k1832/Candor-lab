/* GENERATED C mirror of reference module m190. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S190_0;

static S190_0 mk190_0(long a) {
    S190_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe190_0(const S190_0 *s) {
    return s->a + s->n0;
}
static long read190_0(const S190_0 *s) {
    return s->a * 3;
}
static void bump190_0(S190_0 *s, long d) {
    s->a = s->a + d;
}
static long classify190_0(int tag, long a, long b) {
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
static long accum190_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard190_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S190_1;

static S190_1 mk190_1(long a) {
    S190_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe190_1(const S190_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read190_1(const S190_1 *s) {
    return s->a * 4;
}
static void bump190_1(S190_1 *s, long d) {
    s->a = s->a + d;
}
static long classify190_1(int tag, long a, long b) {
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
static long accum190_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard190_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S190_2;

static S190_2 mk190_2(long a) {
    S190_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe190_2(const S190_2 *s) {
    return s->a + s->n0;
}
static long read190_2(const S190_2 *s) {
    return s->a * 7;
}
static void bump190_2(S190_2 *s, long d) {
    s->a = s->a + d;
}
static long classify190_2(int tag, long a, long b) {
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
static long accum190_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard190_2(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S190_3;

static S190_3 mk190_3(long a) {
    S190_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe190_3(const S190_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read190_3(const S190_3 *s) {
    return s->a * 7;
}
static void bump190_3(S190_3 *s, long d) {
    s->a = s->a + d;
}
static long classify190_3(int tag, long a, long b) {
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
static long accum190_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard190_3(long x) {
    return x + 8;
}

long f190(long x) {
    long acc = x;
    acc += f097(x + 1);
    acc += f158(x + 2);
    acc += f159(x + 3);
    S190_0 s0 = mk190_0(acc);
    bump190_0(&s0, 2);
    acc += probe190_0(&s0);
    acc += read190_0(&s0);
    acc += classify190_0(1, acc, acc);
    acc += accum190_0(4);
    acc += guard190_0(acc);
    S190_1 s1 = mk190_1(acc);
    bump190_1(&s1, 5);
    acc += probe190_1(&s1);
    acc += read190_1(&s1);
    acc += classify190_1(1, acc, acc);
    acc += accum190_1(8);
    acc += guard190_1(acc);
    S190_2 s2 = mk190_2(acc);
    bump190_2(&s2, 4);
    acc += probe190_2(&s2);
    acc += read190_2(&s2);
    acc += classify190_2(1, acc, acc);
    acc += accum190_2(9);
    acc += guard190_2(acc);
    S190_3 s3 = mk190_3(acc);
    bump190_3(&s3, 8);
    acc += probe190_3(&s3);
    acc += read190_3(&s3);
    acc += classify190_3(1, acc, acc);
    acc += accum190_3(6);
    acc += guard190_3(acc);
    return clampi(acc);
}
