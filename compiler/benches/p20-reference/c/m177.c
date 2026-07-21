/* GENERATED C mirror of reference module m177. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S177_0;

static S177_0 mk177_0(long a) {
    S177_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe177_0(const S177_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read177_0(const S177_0 *s) {
    return s->a * 4;
}
static void bump177_0(S177_0 *s, long d) {
    s->a = s->a + d;
}
static long classify177_0(int tag, long a, long b) {
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
static long accum177_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard177_0(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S177_1;

static S177_1 mk177_1(long a) {
    S177_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe177_1(const S177_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read177_1(const S177_1 *s) {
    return s->a * 2;
}
static void bump177_1(S177_1 *s, long d) {
    s->a = s->a + d;
}
static long classify177_1(int tag, long a, long b) {
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
static long accum177_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard177_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S177_2;

static S177_2 mk177_2(long a) {
    S177_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe177_2(const S177_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read177_2(const S177_2 *s) {
    return s->a * 7;
}
static void bump177_2(S177_2 *s, long d) {
    s->a = s->a + d;
}
static long classify177_2(int tag, long a, long b) {
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
static long accum177_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard177_2(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S177_3;

static S177_3 mk177_3(long a) {
    S177_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe177_3(const S177_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read177_3(const S177_3 *s) {
    return s->a * 6;
}
static void bump177_3(S177_3 *s, long d) {
    s->a = s->a + d;
}
static long classify177_3(int tag, long a, long b) {
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
static long accum177_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard177_3(long x) {
    return x + 3;
}

long f177(long x) {
    long acc = x;
    acc += f125(x + 1);
    S177_0 s0 = mk177_0(acc);
    bump177_0(&s0, 3);
    acc += probe177_0(&s0);
    acc += read177_0(&s0);
    acc += classify177_0(1, acc, acc);
    acc += accum177_0(5);
    acc += guard177_0(acc);
    S177_1 s1 = mk177_1(acc);
    bump177_1(&s1, 4);
    acc += probe177_1(&s1);
    acc += read177_1(&s1);
    acc += classify177_1(1, acc, acc);
    acc += accum177_1(7);
    acc += guard177_1(acc);
    S177_2 s2 = mk177_2(acc);
    bump177_2(&s2, 3);
    acc += probe177_2(&s2);
    acc += read177_2(&s2);
    acc += classify177_2(1, acc, acc);
    acc += accum177_2(7);
    acc += guard177_2(acc);
    S177_3 s3 = mk177_3(acc);
    bump177_3(&s3, 9);
    acc += probe177_3(&s3);
    acc += read177_3(&s3);
    acc += classify177_3(1, acc, acc);
    acc += accum177_3(6);
    acc += guard177_3(acc);
    return clampi(acc);
}
