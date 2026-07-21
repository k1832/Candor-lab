/* GENERATED C mirror of reference module m118. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S118_0;

static S118_0 mk118_0(long a) {
    S118_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe118_0(const S118_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read118_0(const S118_0 *s) {
    return s->a * 2;
}
static void bump118_0(S118_0 *s, long d) {
    s->a = s->a + d;
}
static long classify118_0(int tag, long a, long b) {
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
static long accum118_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard118_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S118_1;

static S118_1 mk118_1(long a) {
    S118_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe118_1(const S118_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read118_1(const S118_1 *s) {
    return s->a * 4;
}
static void bump118_1(S118_1 *s, long d) {
    s->a = s->a + d;
}
static long classify118_1(int tag, long a, long b) {
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
static long accum118_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard118_1(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S118_2;

static S118_2 mk118_2(long a) {
    S118_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe118_2(const S118_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read118_2(const S118_2 *s) {
    return s->a * 2;
}
static void bump118_2(S118_2 *s, long d) {
    s->a = s->a + d;
}
static long classify118_2(int tag, long a, long b) {
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
static long accum118_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard118_2(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S118_3;

static S118_3 mk118_3(long a) {
    S118_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe118_3(const S118_3 *s) {
    return s->a + s->n0;
}
static long read118_3(const S118_3 *s) {
    return s->a * 3;
}
static void bump118_3(S118_3 *s, long d) {
    s->a = s->a + d;
}
static long classify118_3(int tag, long a, long b) {
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
static long accum118_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard118_3(long x) {
    return x + 3;
}

long f118(long x) {
    long acc = x;
    acc += f003(x + 1);
    acc += f036(x + 2);
    acc += f038(x + 3);
    acc += f068(x + 4);
    S118_0 s0 = mk118_0(acc);
    bump118_0(&s0, 9);
    acc += probe118_0(&s0);
    acc += read118_0(&s0);
    acc += classify118_0(1, acc, acc);
    acc += accum118_0(8);
    acc += guard118_0(acc);
    S118_1 s1 = mk118_1(acc);
    bump118_1(&s1, 5);
    acc += probe118_1(&s1);
    acc += read118_1(&s1);
    acc += classify118_1(1, acc, acc);
    acc += accum118_1(3);
    acc += guard118_1(acc);
    S118_2 s2 = mk118_2(acc);
    bump118_2(&s2, 6);
    acc += probe118_2(&s2);
    acc += read118_2(&s2);
    acc += classify118_2(1, acc, acc);
    acc += accum118_2(8);
    acc += guard118_2(acc);
    S118_3 s3 = mk118_3(acc);
    bump118_3(&s3, 5);
    acc += probe118_3(&s3);
    acc += read118_3(&s3);
    acc += classify118_3(1, acc, acc);
    acc += accum118_3(7);
    acc += guard118_3(acc);
    return clampi(acc);
}
