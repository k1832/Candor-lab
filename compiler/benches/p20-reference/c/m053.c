/* GENERATED C mirror of reference module m053. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S53_0;

static S53_0 mk53_0(long a) {
    S53_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe53_0(const S53_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read53_0(const S53_0 *s) {
    return s->a * 5;
}
static void bump53_0(S53_0 *s, long d) {
    s->a = s->a + d;
}
static long classify53_0(int tag, long a, long b) {
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
static long accum53_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard53_0(long x) {
    return x + 4;
}

static long pick53_0_0(long a, long b) { return a > b ? a : b; }
static long pick53_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S53_1;

static S53_1 mk53_1(long a) {
    S53_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe53_1(const S53_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read53_1(const S53_1 *s) {
    return s->a * 5;
}
static void bump53_1(S53_1 *s, long d) {
    s->a = s->a + d;
}
static long classify53_1(int tag, long a, long b) {
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
static long accum53_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard53_1(long x) {
    return x + 1;
}

static long pick53_1_0(long a, long b) { return a > b ? a : b; }
static long pick53_1_1(long a, long b) { return a > b ? a : b; }
static long pick53_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S53_2;

static S53_2 mk53_2(long a) {
    S53_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe53_2(const S53_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read53_2(const S53_2 *s) {
    return s->a * 6;
}
static void bump53_2(S53_2 *s, long d) {
    s->a = s->a + d;
}
static long classify53_2(int tag, long a, long b) {
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
static long accum53_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard53_2(long x) {
    return x + 9;
}

static long pick53_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S53_3;

static S53_3 mk53_3(long a) {
    S53_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe53_3(const S53_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read53_3(const S53_3 *s) {
    return s->a * 3;
}
static void bump53_3(S53_3 *s, long d) {
    s->a = s->a + d;
}
static long classify53_3(int tag, long a, long b) {
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
static long accum53_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard53_3(long x) {
    return x + 4;
}

static long pick53_3_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S53_4;

static S53_4 mk53_4(long a) {
    S53_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe53_4(const S53_4 *s) {
    return s->a + s->n0;
}
static long read53_4(const S53_4 *s) {
    return s->a * 3;
}
static void bump53_4(S53_4 *s, long d) {
    s->a = s->a + d;
}
static long classify53_4(int tag, long a, long b) {
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
static long accum53_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard53_4(long x) {
    return x + 7;
}

static long pick53_4_0(long a, long b) { return a > b ? a : b; }
static long pick53_4_1(long a, long b) { return a > b ? a : b; }
static long pick53_4_2(long a, long b) { return a > b ? a : b; }
long f053(long x) {
    long acc = x;
    acc += f002(x + 1);
    acc += f016(x + 2);
    acc += f020(x + 3);
    acc += f022(x + 4);
    S53_0 s0 = mk53_0(acc);
    bump53_0(&s0, 9);
    acc += probe53_0(&s0);
    acc += read53_0(&s0);
    acc += classify53_0(1, acc, acc);
    acc += accum53_0(3);
    acc += guard53_0(acc);
    acc += pick53_0_0(acc, acc + 1);
    acc += pick53_0_1(acc, acc + 2);
    S53_1 s1 = mk53_1(acc);
    bump53_1(&s1, 4);
    acc += probe53_1(&s1);
    acc += read53_1(&s1);
    acc += classify53_1(1, acc, acc);
    acc += accum53_1(3);
    acc += guard53_1(acc);
    acc += pick53_1_0(acc, acc + 8);
    acc += pick53_1_1(acc, acc + 8);
    acc += pick53_1_2(acc, acc + 9);
    S53_2 s2 = mk53_2(acc);
    bump53_2(&s2, 4);
    acc += probe53_2(&s2);
    acc += read53_2(&s2);
    acc += classify53_2(1, acc, acc);
    acc += accum53_2(3);
    acc += guard53_2(acc);
    acc += pick53_2_0(acc, acc + 1);
    S53_3 s3 = mk53_3(acc);
    bump53_3(&s3, 5);
    acc += probe53_3(&s3);
    acc += read53_3(&s3);
    acc += classify53_3(1, acc, acc);
    acc += accum53_3(4);
    acc += guard53_3(acc);
    acc += pick53_3_0(acc, acc + 2);
    S53_4 s4 = mk53_4(acc);
    bump53_4(&s4, 1);
    acc += probe53_4(&s4);
    acc += read53_4(&s4);
    acc += classify53_4(1, acc, acc);
    acc += accum53_4(5);
    acc += guard53_4(acc);
    acc += pick53_4_0(acc, acc + 5);
    acc += pick53_4_1(acc, acc + 7);
    acc += pick53_4_2(acc, acc + 4);
    return clampi(acc);
}
