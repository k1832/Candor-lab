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
    return s->a * 4;
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
    return x + 1;
}

static long pick118_0_0(long a, long b) { return a > b ? a : b; }
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
    return s->a * 2;
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
        acc += i * 4;
    }
    return acc;
}
static long guard118_1(long x) {
    return x + 4;
}

static long pick118_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S118_2;

static S118_2 mk118_2(long a) {
    S118_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe118_2(const S118_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read118_2(const S118_2 *s) {
    return s->a * 7;
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
    return x + 3;
}

static long pick118_2_0(long a, long b) { return a > b ? a : b; }
static long pick118_2_1(long a, long b) { return a > b ? a : b; }
static long pick118_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S118_3;

static S118_3 mk118_3(long a) {
    S118_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe118_3(const S118_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read118_3(const S118_3 *s) {
    return s->a * 4;
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
        acc += i * 2;
    }
    return acc;
}
static long guard118_3(long x) {
    return x + 3;
}

static long pick118_3_0(long a, long b) { return a > b ? a : b; }
static long pick118_3_1(long a, long b) { return a > b ? a : b; }
static long pick118_3_2(long a, long b) { return a > b ? a : b; }
long f118(long x) {
    long acc = x;
    acc += f031(x + 1);
    acc += f105(x + 2);
    S118_0 s0 = mk118_0(acc);
    bump118_0(&s0, 9);
    acc += probe118_0(&s0);
    acc += read118_0(&s0);
    acc += classify118_0(1, acc, acc);
    acc += accum118_0(3);
    acc += guard118_0(acc);
    acc += pick118_0_0(acc, acc + 9);
    S118_1 s1 = mk118_1(acc);
    bump118_1(&s1, 9);
    acc += probe118_1(&s1);
    acc += read118_1(&s1);
    acc += classify118_1(1, acc, acc);
    acc += accum118_1(9);
    acc += guard118_1(acc);
    acc += pick118_1_0(acc, acc + 3);
    S118_2 s2 = mk118_2(acc);
    bump118_2(&s2, 7);
    acc += probe118_2(&s2);
    acc += read118_2(&s2);
    acc += classify118_2(1, acc, acc);
    acc += accum118_2(6);
    acc += guard118_2(acc);
    acc += pick118_2_0(acc, acc + 5);
    acc += pick118_2_1(acc, acc + 3);
    acc += pick118_2_2(acc, acc + 5);
    S118_3 s3 = mk118_3(acc);
    bump118_3(&s3, 9);
    acc += probe118_3(&s3);
    acc += read118_3(&s3);
    acc += classify118_3(1, acc, acc);
    acc += accum118_3(3);
    acc += guard118_3(acc);
    acc += pick118_3_0(acc, acc + 6);
    acc += pick118_3_1(acc, acc + 1);
    acc += pick118_3_2(acc, acc + 5);
    return clampi(acc);
}
