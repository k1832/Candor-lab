/* GENERATED C mirror of reference module m020. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S20_0;

static S20_0 mk20_0(long a) {
    S20_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe20_0(const S20_0 *s) {
    return s->a + s->n0;
}
static long read20_0(const S20_0 *s) {
    return s->a * 4;
}
static void bump20_0(S20_0 *s, long d) {
    s->a = s->a + d;
}
static long classify20_0(int tag, long a, long b) {
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
static long accum20_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard20_0(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S20_1;

static S20_1 mk20_1(long a) {
    S20_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe20_1(const S20_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read20_1(const S20_1 *s) {
    return s->a * 5;
}
static void bump20_1(S20_1 *s, long d) {
    s->a = s->a + d;
}
static long classify20_1(int tag, long a, long b) {
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
static long accum20_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard20_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S20_2;

static S20_2 mk20_2(long a) {
    S20_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe20_2(const S20_2 *s) {
    return s->a + s->n0;
}
static long read20_2(const S20_2 *s) {
    return s->a * 3;
}
static void bump20_2(S20_2 *s, long d) {
    s->a = s->a + d;
}
static long classify20_2(int tag, long a, long b) {
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
static long accum20_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard20_2(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S20_3;

static S20_3 mk20_3(long a) {
    S20_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe20_3(const S20_3 *s) {
    return s->a + s->n0;
}
static long read20_3(const S20_3 *s) {
    return s->a * 5;
}
static void bump20_3(S20_3 *s, long d) {
    s->a = s->a + d;
}
static long classify20_3(int tag, long a, long b) {
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
static long accum20_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard20_3(long x) {
    return x + 4;
}

long f020(long x) {
    long acc = x;
    acc += f000(x + 1);
    S20_0 s0 = mk20_0(acc);
    bump20_0(&s0, 7);
    acc += probe20_0(&s0);
    acc += read20_0(&s0);
    acc += classify20_0(1, acc, acc);
    acc += accum20_0(6);
    acc += guard20_0(acc);
    S20_1 s1 = mk20_1(acc);
    bump20_1(&s1, 3);
    acc += probe20_1(&s1);
    acc += read20_1(&s1);
    acc += classify20_1(1, acc, acc);
    acc += accum20_1(3);
    acc += guard20_1(acc);
    S20_2 s2 = mk20_2(acc);
    bump20_2(&s2, 8);
    acc += probe20_2(&s2);
    acc += read20_2(&s2);
    acc += classify20_2(1, acc, acc);
    acc += accum20_2(6);
    acc += guard20_2(acc);
    S20_3 s3 = mk20_3(acc);
    bump20_3(&s3, 3);
    acc += probe20_3(&s3);
    acc += read20_3(&s3);
    acc += classify20_3(1, acc, acc);
    acc += accum20_3(4);
    acc += guard20_3(acc);
    return clampi(acc);
}
