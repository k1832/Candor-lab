/* GENERATED C mirror of reference module m015. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S15_0;

static S15_0 mk15_0(long a) {
    S15_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe15_0(const S15_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read15_0(const S15_0 *s) {
    return s->a * 2;
}
static void bump15_0(S15_0 *s, long d) {
    s->a = s->a + d;
}
static long classify15_0(int tag, long a, long b) {
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
static long accum15_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard15_0(long x) {
    return x + 2;
}

static long pick15_0_0(long a, long b) { return a > b ? a : b; }
static long pick15_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S15_1;

static S15_1 mk15_1(long a) {
    S15_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe15_1(const S15_1 *s) {
    return s->a + s->n0;
}
static long read15_1(const S15_1 *s) {
    return s->a * 5;
}
static void bump15_1(S15_1 *s, long d) {
    s->a = s->a + d;
}
static long classify15_1(int tag, long a, long b) {
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
static long accum15_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard15_1(long x) {
    return x + 9;
}

static long pick15_1_0(long a, long b) { return a > b ? a : b; }
static long pick15_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S15_2;

static S15_2 mk15_2(long a) {
    S15_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe15_2(const S15_2 *s) {
    return s->a + s->n0;
}
static long read15_2(const S15_2 *s) {
    return s->a * 2;
}
static void bump15_2(S15_2 *s, long d) {
    s->a = s->a + d;
}
static long classify15_2(int tag, long a, long b) {
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
static long accum15_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard15_2(long x) {
    return x + 3;
}

static long pick15_2_0(long a, long b) { return a > b ? a : b; }
static long pick15_2_1(long a, long b) { return a > b ? a : b; }
static long pick15_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S15_3;

static S15_3 mk15_3(long a) {
    S15_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe15_3(const S15_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read15_3(const S15_3 *s) {
    return s->a * 3;
}
static void bump15_3(S15_3 *s, long d) {
    s->a = s->a + d;
}
static long classify15_3(int tag, long a, long b) {
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
static long accum15_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard15_3(long x) {
    return x + 3;
}

static long pick15_3_0(long a, long b) { return a > b ? a : b; }
static long pick15_3_1(long a, long b) { return a > b ? a : b; }
static long pick15_3_2(long a, long b) { return a > b ? a : b; }
long f015(long x) {
    long acc = x;
    acc += f002(x + 1);
    acc += f007(x + 2);
    S15_0 s0 = mk15_0(acc);
    bump15_0(&s0, 3);
    acc += probe15_0(&s0);
    acc += read15_0(&s0);
    acc += classify15_0(1, acc, acc);
    acc += accum15_0(3);
    acc += guard15_0(acc);
    acc += pick15_0_0(acc, acc + 6);
    acc += pick15_0_1(acc, acc + 2);
    S15_1 s1 = mk15_1(acc);
    bump15_1(&s1, 8);
    acc += probe15_1(&s1);
    acc += read15_1(&s1);
    acc += classify15_1(1, acc, acc);
    acc += accum15_1(4);
    acc += guard15_1(acc);
    acc += pick15_1_0(acc, acc + 1);
    acc += pick15_1_1(acc, acc + 2);
    S15_2 s2 = mk15_2(acc);
    bump15_2(&s2, 5);
    acc += probe15_2(&s2);
    acc += read15_2(&s2);
    acc += classify15_2(1, acc, acc);
    acc += accum15_2(7);
    acc += guard15_2(acc);
    acc += pick15_2_0(acc, acc + 8);
    acc += pick15_2_1(acc, acc + 1);
    acc += pick15_2_2(acc, acc + 3);
    S15_3 s3 = mk15_3(acc);
    bump15_3(&s3, 5);
    acc += probe15_3(&s3);
    acc += read15_3(&s3);
    acc += classify15_3(1, acc, acc);
    acc += accum15_3(4);
    acc += guard15_3(acc);
    acc += pick15_3_0(acc, acc + 6);
    acc += pick15_3_1(acc, acc + 3);
    acc += pick15_3_2(acc, acc + 6);
    return clampi(acc);
}
