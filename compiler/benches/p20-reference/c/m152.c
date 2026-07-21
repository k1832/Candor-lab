/* GENERATED C mirror of reference module m152. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S152_0;

static S152_0 mk152_0(long a) {
    S152_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe152_0(const S152_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read152_0(const S152_0 *s) {
    return s->a * 7;
}
static void bump152_0(S152_0 *s, long d) {
    s->a = s->a + d;
}
static long classify152_0(int tag, long a, long b) {
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
static long accum152_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard152_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S152_1;

static S152_1 mk152_1(long a) {
    S152_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe152_1(const S152_1 *s) {
    return s->a + s->n0;
}
static long read152_1(const S152_1 *s) {
    return s->a * 5;
}
static void bump152_1(S152_1 *s, long d) {
    s->a = s->a + d;
}
static long classify152_1(int tag, long a, long b) {
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
static long accum152_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard152_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S152_2;

static S152_2 mk152_2(long a) {
    S152_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe152_2(const S152_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read152_2(const S152_2 *s) {
    return s->a * 5;
}
static void bump152_2(S152_2 *s, long d) {
    s->a = s->a + d;
}
static long classify152_2(int tag, long a, long b) {
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
static long accum152_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard152_2(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S152_3;

static S152_3 mk152_3(long a) {
    S152_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe152_3(const S152_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read152_3(const S152_3 *s) {
    return s->a * 2;
}
static void bump152_3(S152_3 *s, long d) {
    s->a = s->a + d;
}
static long classify152_3(int tag, long a, long b) {
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
static long accum152_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard152_3(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S152_4;

static S152_4 mk152_4(long a) {
    S152_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe152_4(const S152_4 *s) {
    return s->a + s->n0;
}
static long read152_4(const S152_4 *s) {
    return s->a * 5;
}
static void bump152_4(S152_4 *s, long d) {
    s->a = s->a + d;
}
static long classify152_4(int tag, long a, long b) {
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
static long accum152_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard152_4(long x) {
    return x + 5;
}

long f152(long x) {
    long acc = x;
    acc += f015(x + 1);
    acc += f119(x + 2);
    S152_0 s0 = mk152_0(acc);
    bump152_0(&s0, 8);
    acc += probe152_0(&s0);
    acc += read152_0(&s0);
    acc += classify152_0(1, acc, acc);
    acc += accum152_0(8);
    acc += guard152_0(acc);
    S152_1 s1 = mk152_1(acc);
    bump152_1(&s1, 9);
    acc += probe152_1(&s1);
    acc += read152_1(&s1);
    acc += classify152_1(1, acc, acc);
    acc += accum152_1(6);
    acc += guard152_1(acc);
    S152_2 s2 = mk152_2(acc);
    bump152_2(&s2, 1);
    acc += probe152_2(&s2);
    acc += read152_2(&s2);
    acc += classify152_2(1, acc, acc);
    acc += accum152_2(4);
    acc += guard152_2(acc);
    S152_3 s3 = mk152_3(acc);
    bump152_3(&s3, 5);
    acc += probe152_3(&s3);
    acc += read152_3(&s3);
    acc += classify152_3(1, acc, acc);
    acc += accum152_3(6);
    acc += guard152_3(acc);
    S152_4 s4 = mk152_4(acc);
    bump152_4(&s4, 8);
    acc += probe152_4(&s4);
    acc += read152_4(&s4);
    acc += classify152_4(1, acc, acc);
    acc += accum152_4(8);
    acc += guard152_4(acc);
    return clampi(acc);
}
