/* GENERATED C mirror of reference module m004. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S4_0;

static S4_0 mk4_0(long a) {
    S4_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe4_0(const S4_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read4_0(const S4_0 *s) {
    return s->a * 5;
}
static void bump4_0(S4_0 *s, long d) {
    s->a = s->a + d;
}
static long classify4_0(int tag, long a, long b) {
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
static long accum4_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard4_0(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S4_1;

static S4_1 mk4_1(long a) {
    S4_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe4_1(const S4_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read4_1(const S4_1 *s) {
    return s->a * 6;
}
static void bump4_1(S4_1 *s, long d) {
    s->a = s->a + d;
}
static long classify4_1(int tag, long a, long b) {
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
static long accum4_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard4_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S4_2;

static S4_2 mk4_2(long a) {
    S4_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe4_2(const S4_2 *s) {
    return s->a + s->n0;
}
static long read4_2(const S4_2 *s) {
    return s->a * 5;
}
static void bump4_2(S4_2 *s, long d) {
    s->a = s->a + d;
}
static long classify4_2(int tag, long a, long b) {
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
static long accum4_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard4_2(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S4_3;

static S4_3 mk4_3(long a) {
    S4_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe4_3(const S4_3 *s) {
    return s->a + s->n0;
}
static long read4_3(const S4_3 *s) {
    return s->a * 4;
}
static void bump4_3(S4_3 *s, long d) {
    s->a = s->a + d;
}
static long classify4_3(int tag, long a, long b) {
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
static long accum4_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard4_3(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S4_4;

static S4_4 mk4_4(long a) {
    S4_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe4_4(const S4_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read4_4(const S4_4 *s) {
    return s->a * 7;
}
static void bump4_4(S4_4 *s, long d) {
    s->a = s->a + d;
}
static long classify4_4(int tag, long a, long b) {
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
static long accum4_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard4_4(long x) {
    return x + 1;
}

long f004(long x) {
    long acc = x;
    S4_0 s0 = mk4_0(acc);
    bump4_0(&s0, 8);
    acc += probe4_0(&s0);
    acc += read4_0(&s0);
    acc += classify4_0(1, acc, acc);
    acc += accum4_0(3);
    acc += guard4_0(acc);
    S4_1 s1 = mk4_1(acc);
    bump4_1(&s1, 4);
    acc += probe4_1(&s1);
    acc += read4_1(&s1);
    acc += classify4_1(1, acc, acc);
    acc += accum4_1(5);
    acc += guard4_1(acc);
    S4_2 s2 = mk4_2(acc);
    bump4_2(&s2, 4);
    acc += probe4_2(&s2);
    acc += read4_2(&s2);
    acc += classify4_2(1, acc, acc);
    acc += accum4_2(9);
    acc += guard4_2(acc);
    S4_3 s3 = mk4_3(acc);
    bump4_3(&s3, 8);
    acc += probe4_3(&s3);
    acc += read4_3(&s3);
    acc += classify4_3(1, acc, acc);
    acc += accum4_3(4);
    acc += guard4_3(acc);
    S4_4 s4 = mk4_4(acc);
    bump4_4(&s4, 9);
    acc += probe4_4(&s4);
    acc += read4_4(&s4);
    acc += classify4_4(1, acc, acc);
    acc += accum4_4(5);
    acc += guard4_4(acc);
    return clampi(acc);
}
