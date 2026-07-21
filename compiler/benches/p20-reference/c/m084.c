/* GENERATED C mirror of reference module m084. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S84_0;

static S84_0 mk84_0(long a) {
    S84_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe84_0(const S84_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read84_0(const S84_0 *s) {
    return s->a * 4;
}
static void bump84_0(S84_0 *s, long d) {
    s->a = s->a + d;
}
static long classify84_0(int tag, long a, long b) {
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
static long accum84_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard84_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S84_1;

static S84_1 mk84_1(long a) {
    S84_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe84_1(const S84_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read84_1(const S84_1 *s) {
    return s->a * 3;
}
static void bump84_1(S84_1 *s, long d) {
    s->a = s->a + d;
}
static long classify84_1(int tag, long a, long b) {
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
static long accum84_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard84_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S84_2;

static S84_2 mk84_2(long a) {
    S84_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe84_2(const S84_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read84_2(const S84_2 *s) {
    return s->a * 7;
}
static void bump84_2(S84_2 *s, long d) {
    s->a = s->a + d;
}
static long classify84_2(int tag, long a, long b) {
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
static long accum84_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard84_2(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S84_3;

static S84_3 mk84_3(long a) {
    S84_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe84_3(const S84_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read84_3(const S84_3 *s) {
    return s->a * 4;
}
static void bump84_3(S84_3 *s, long d) {
    s->a = s->a + d;
}
static long classify84_3(int tag, long a, long b) {
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
static long accum84_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard84_3(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S84_4;

static S84_4 mk84_4(long a) {
    S84_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe84_4(const S84_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read84_4(const S84_4 *s) {
    return s->a * 3;
}
static void bump84_4(S84_4 *s, long d) {
    s->a = s->a + d;
}
static long classify84_4(int tag, long a, long b) {
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
static long accum84_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard84_4(long x) {
    return x + 2;
}

long f084(long x) {
    long acc = x;
    acc += f026(x + 1);
    acc += f039(x + 2);
    acc += f070(x + 3);
    S84_0 s0 = mk84_0(acc);
    bump84_0(&s0, 7);
    acc += probe84_0(&s0);
    acc += read84_0(&s0);
    acc += classify84_0(1, acc, acc);
    acc += accum84_0(8);
    acc += guard84_0(acc);
    S84_1 s1 = mk84_1(acc);
    bump84_1(&s1, 6);
    acc += probe84_1(&s1);
    acc += read84_1(&s1);
    acc += classify84_1(1, acc, acc);
    acc += accum84_1(3);
    acc += guard84_1(acc);
    S84_2 s2 = mk84_2(acc);
    bump84_2(&s2, 6);
    acc += probe84_2(&s2);
    acc += read84_2(&s2);
    acc += classify84_2(1, acc, acc);
    acc += accum84_2(5);
    acc += guard84_2(acc);
    S84_3 s3 = mk84_3(acc);
    bump84_3(&s3, 4);
    acc += probe84_3(&s3);
    acc += read84_3(&s3);
    acc += classify84_3(1, acc, acc);
    acc += accum84_3(3);
    acc += guard84_3(acc);
    S84_4 s4 = mk84_4(acc);
    bump84_4(&s4, 2);
    acc += probe84_4(&s4);
    acc += read84_4(&s4);
    acc += classify84_4(1, acc, acc);
    acc += accum84_4(4);
    acc += guard84_4(acc);
    return clampi(acc);
}
