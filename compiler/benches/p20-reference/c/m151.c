/* GENERATED C mirror of reference module m151. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S151_0;

static S151_0 mk151_0(long a) {
    S151_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe151_0(const S151_0 *s) {
    return s->a + s->n0;
}
static long read151_0(const S151_0 *s) {
    return s->a * 4;
}
static void bump151_0(S151_0 *s, long d) {
    s->a = s->a + d;
}
static long classify151_0(int tag, long a, long b) {
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
static long accum151_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard151_0(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S151_1;

static S151_1 mk151_1(long a) {
    S151_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe151_1(const S151_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read151_1(const S151_1 *s) {
    return s->a * 7;
}
static void bump151_1(S151_1 *s, long d) {
    s->a = s->a + d;
}
static long classify151_1(int tag, long a, long b) {
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
static long accum151_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard151_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S151_2;

static S151_2 mk151_2(long a) {
    S151_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe151_2(const S151_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read151_2(const S151_2 *s) {
    return s->a * 6;
}
static void bump151_2(S151_2 *s, long d) {
    s->a = s->a + d;
}
static long classify151_2(int tag, long a, long b) {
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
static long accum151_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard151_2(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S151_3;

static S151_3 mk151_3(long a) {
    S151_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe151_3(const S151_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read151_3(const S151_3 *s) {
    return s->a * 7;
}
static void bump151_3(S151_3 *s, long d) {
    s->a = s->a + d;
}
static long classify151_3(int tag, long a, long b) {
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
static long accum151_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard151_3(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S151_4;

static S151_4 mk151_4(long a) {
    S151_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe151_4(const S151_4 *s) {
    return s->a + s->n0;
}
static long read151_4(const S151_4 *s) {
    return s->a * 5;
}
static void bump151_4(S151_4 *s, long d) {
    s->a = s->a + d;
}
static long classify151_4(int tag, long a, long b) {
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
static long accum151_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard151_4(long x) {
    return x + 2;
}

long f151(long x) {
    long acc = x;
    acc += f112(x + 1);
    S151_0 s0 = mk151_0(acc);
    bump151_0(&s0, 9);
    acc += probe151_0(&s0);
    acc += read151_0(&s0);
    acc += classify151_0(1, acc, acc);
    acc += accum151_0(6);
    acc += guard151_0(acc);
    S151_1 s1 = mk151_1(acc);
    bump151_1(&s1, 4);
    acc += probe151_1(&s1);
    acc += read151_1(&s1);
    acc += classify151_1(1, acc, acc);
    acc += accum151_1(9);
    acc += guard151_1(acc);
    S151_2 s2 = mk151_2(acc);
    bump151_2(&s2, 9);
    acc += probe151_2(&s2);
    acc += read151_2(&s2);
    acc += classify151_2(1, acc, acc);
    acc += accum151_2(9);
    acc += guard151_2(acc);
    S151_3 s3 = mk151_3(acc);
    bump151_3(&s3, 4);
    acc += probe151_3(&s3);
    acc += read151_3(&s3);
    acc += classify151_3(1, acc, acc);
    acc += accum151_3(3);
    acc += guard151_3(acc);
    S151_4 s4 = mk151_4(acc);
    bump151_4(&s4, 9);
    acc += probe151_4(&s4);
    acc += read151_4(&s4);
    acc += classify151_4(1, acc, acc);
    acc += accum151_4(3);
    acc += guard151_4(acc);
    return clampi(acc);
}
