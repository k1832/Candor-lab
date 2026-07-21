/* GENERATED C mirror of reference module m154. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S154_0;

static S154_0 mk154_0(long a) {
    S154_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe154_0(const S154_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read154_0(const S154_0 *s) {
    return s->a * 7;
}
static void bump154_0(S154_0 *s, long d) {
    s->a = s->a + d;
}
static long classify154_0(int tag, long a, long b) {
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
static long accum154_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard154_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S154_1;

static S154_1 mk154_1(long a) {
    S154_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe154_1(const S154_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read154_1(const S154_1 *s) {
    return s->a * 4;
}
static void bump154_1(S154_1 *s, long d) {
    s->a = s->a + d;
}
static long classify154_1(int tag, long a, long b) {
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
static long accum154_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard154_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S154_2;

static S154_2 mk154_2(long a) {
    S154_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe154_2(const S154_2 *s) {
    return s->a + s->n0;
}
static long read154_2(const S154_2 *s) {
    return s->a * 4;
}
static void bump154_2(S154_2 *s, long d) {
    s->a = s->a + d;
}
static long classify154_2(int tag, long a, long b) {
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
static long accum154_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard154_2(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S154_3;

static S154_3 mk154_3(long a) {
    S154_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe154_3(const S154_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read154_3(const S154_3 *s) {
    return s->a * 5;
}
static void bump154_3(S154_3 *s, long d) {
    s->a = s->a + d;
}
static long classify154_3(int tag, long a, long b) {
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
static long accum154_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard154_3(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S154_4;

static S154_4 mk154_4(long a) {
    S154_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe154_4(const S154_4 *s) {
    return s->a + s->n0;
}
static long read154_4(const S154_4 *s) {
    return s->a * 6;
}
static void bump154_4(S154_4 *s, long d) {
    s->a = s->a + d;
}
static long classify154_4(int tag, long a, long b) {
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
static long accum154_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard154_4(long x) {
    return x + 1;
}

long f154(long x) {
    long acc = x;
    acc += f088(x + 1);
    acc += f123(x + 2);
    S154_0 s0 = mk154_0(acc);
    bump154_0(&s0, 2);
    acc += probe154_0(&s0);
    acc += read154_0(&s0);
    acc += classify154_0(1, acc, acc);
    acc += accum154_0(9);
    acc += guard154_0(acc);
    S154_1 s1 = mk154_1(acc);
    bump154_1(&s1, 4);
    acc += probe154_1(&s1);
    acc += read154_1(&s1);
    acc += classify154_1(1, acc, acc);
    acc += accum154_1(7);
    acc += guard154_1(acc);
    S154_2 s2 = mk154_2(acc);
    bump154_2(&s2, 4);
    acc += probe154_2(&s2);
    acc += read154_2(&s2);
    acc += classify154_2(1, acc, acc);
    acc += accum154_2(5);
    acc += guard154_2(acc);
    S154_3 s3 = mk154_3(acc);
    bump154_3(&s3, 2);
    acc += probe154_3(&s3);
    acc += read154_3(&s3);
    acc += classify154_3(1, acc, acc);
    acc += accum154_3(3);
    acc += guard154_3(acc);
    S154_4 s4 = mk154_4(acc);
    bump154_4(&s4, 2);
    acc += probe154_4(&s4);
    acc += read154_4(&s4);
    acc += classify154_4(1, acc, acc);
    acc += accum154_4(8);
    acc += guard154_4(acc);
    return clampi(acc);
}
