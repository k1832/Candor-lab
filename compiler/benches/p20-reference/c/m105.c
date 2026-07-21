/* GENERATED C mirror of reference module m105. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S105_0;

static S105_0 mk105_0(long a) {
    S105_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe105_0(const S105_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read105_0(const S105_0 *s) {
    return s->a * 3;
}
static void bump105_0(S105_0 *s, long d) {
    s->a = s->a + d;
}
static long classify105_0(int tag, long a, long b) {
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
static long accum105_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard105_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S105_1;

static S105_1 mk105_1(long a) {
    S105_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe105_1(const S105_1 *s) {
    return s->a + s->n0;
}
static long read105_1(const S105_1 *s) {
    return s->a * 5;
}
static void bump105_1(S105_1 *s, long d) {
    s->a = s->a + d;
}
static long classify105_1(int tag, long a, long b) {
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
static long accum105_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard105_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S105_2;

static S105_2 mk105_2(long a) {
    S105_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe105_2(const S105_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read105_2(const S105_2 *s) {
    return s->a * 4;
}
static void bump105_2(S105_2 *s, long d) {
    s->a = s->a + d;
}
static long classify105_2(int tag, long a, long b) {
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
static long accum105_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard105_2(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S105_3;

static S105_3 mk105_3(long a) {
    S105_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe105_3(const S105_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read105_3(const S105_3 *s) {
    return s->a * 7;
}
static void bump105_3(S105_3 *s, long d) {
    s->a = s->a + d;
}
static long classify105_3(int tag, long a, long b) {
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
static long accum105_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard105_3(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S105_4;

static S105_4 mk105_4(long a) {
    S105_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe105_4(const S105_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read105_4(const S105_4 *s) {
    return s->a * 5;
}
static void bump105_4(S105_4 *s, long d) {
    s->a = s->a + d;
}
static long classify105_4(int tag, long a, long b) {
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
static long accum105_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard105_4(long x) {
    return x + 7;
}

long f105(long x) {
    long acc = x;
    acc += f002(x + 1);
    acc += f029(x + 2);
    acc += f046(x + 3);
    S105_0 s0 = mk105_0(acc);
    bump105_0(&s0, 8);
    acc += probe105_0(&s0);
    acc += read105_0(&s0);
    acc += classify105_0(1, acc, acc);
    acc += accum105_0(9);
    acc += guard105_0(acc);
    S105_1 s1 = mk105_1(acc);
    bump105_1(&s1, 2);
    acc += probe105_1(&s1);
    acc += read105_1(&s1);
    acc += classify105_1(1, acc, acc);
    acc += accum105_1(8);
    acc += guard105_1(acc);
    S105_2 s2 = mk105_2(acc);
    bump105_2(&s2, 9);
    acc += probe105_2(&s2);
    acc += read105_2(&s2);
    acc += classify105_2(1, acc, acc);
    acc += accum105_2(3);
    acc += guard105_2(acc);
    S105_3 s3 = mk105_3(acc);
    bump105_3(&s3, 8);
    acc += probe105_3(&s3);
    acc += read105_3(&s3);
    acc += classify105_3(1, acc, acc);
    acc += accum105_3(8);
    acc += guard105_3(acc);
    S105_4 s4 = mk105_4(acc);
    bump105_4(&s4, 4);
    acc += probe105_4(&s4);
    acc += read105_4(&s4);
    acc += classify105_4(1, acc, acc);
    acc += accum105_4(5);
    acc += guard105_4(acc);
    return clampi(acc);
}
