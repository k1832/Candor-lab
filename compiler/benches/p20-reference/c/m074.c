/* GENERATED C mirror of reference module m074. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S74_0;

static S74_0 mk74_0(long a) {
    S74_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe74_0(const S74_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read74_0(const S74_0 *s) {
    return s->a * 2;
}
static void bump74_0(S74_0 *s, long d) {
    s->a = s->a + d;
}
static long classify74_0(int tag, long a, long b) {
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
static long accum74_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard74_0(long x) {
    return x + 7;
}

static long pick74_0_0(long a, long b) { return a > b ? a : b; }
static long pick74_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S74_1;

static S74_1 mk74_1(long a) {
    S74_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe74_1(const S74_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read74_1(const S74_1 *s) {
    return s->a * 4;
}
static void bump74_1(S74_1 *s, long d) {
    s->a = s->a + d;
}
static long classify74_1(int tag, long a, long b) {
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
static long accum74_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard74_1(long x) {
    return x + 8;
}

static long pick74_1_0(long a, long b) { return a > b ? a : b; }
static long pick74_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S74_2;

static S74_2 mk74_2(long a) {
    S74_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe74_2(const S74_2 *s) {
    return s->a + s->n0;
}
static long read74_2(const S74_2 *s) {
    return s->a * 3;
}
static void bump74_2(S74_2 *s, long d) {
    s->a = s->a + d;
}
static long classify74_2(int tag, long a, long b) {
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
static long accum74_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard74_2(long x) {
    return x + 6;
}

static long pick74_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S74_3;

static S74_3 mk74_3(long a) {
    S74_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe74_3(const S74_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read74_3(const S74_3 *s) {
    return s->a * 6;
}
static void bump74_3(S74_3 *s, long d) {
    s->a = s->a + d;
}
static long classify74_3(int tag, long a, long b) {
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
static long accum74_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard74_3(long x) {
    return x + 9;
}

static long pick74_3_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S74_4;

static S74_4 mk74_4(long a) {
    S74_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe74_4(const S74_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read74_4(const S74_4 *s) {
    return s->a * 2;
}
static void bump74_4(S74_4 *s, long d) {
    s->a = s->a + d;
}
static long classify74_4(int tag, long a, long b) {
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
static long accum74_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard74_4(long x) {
    return x + 8;
}

static long pick74_4_0(long a, long b) { return a > b ? a : b; }
static long pick74_4_1(long a, long b) { return a > b ? a : b; }
static long pick74_4_2(long a, long b) { return a > b ? a : b; }
long f074(long x) {
    long acc = x;
    acc += f009(x + 1);
    S74_0 s0 = mk74_0(acc);
    bump74_0(&s0, 4);
    acc += probe74_0(&s0);
    acc += read74_0(&s0);
    acc += classify74_0(1, acc, acc);
    acc += accum74_0(8);
    acc += guard74_0(acc);
    acc += pick74_0_0(acc, acc + 6);
    acc += pick74_0_1(acc, acc + 6);
    S74_1 s1 = mk74_1(acc);
    bump74_1(&s1, 6);
    acc += probe74_1(&s1);
    acc += read74_1(&s1);
    acc += classify74_1(1, acc, acc);
    acc += accum74_1(9);
    acc += guard74_1(acc);
    acc += pick74_1_0(acc, acc + 4);
    acc += pick74_1_1(acc, acc + 2);
    S74_2 s2 = mk74_2(acc);
    bump74_2(&s2, 4);
    acc += probe74_2(&s2);
    acc += read74_2(&s2);
    acc += classify74_2(1, acc, acc);
    acc += accum74_2(4);
    acc += guard74_2(acc);
    acc += pick74_2_0(acc, acc + 4);
    S74_3 s3 = mk74_3(acc);
    bump74_3(&s3, 1);
    acc += probe74_3(&s3);
    acc += read74_3(&s3);
    acc += classify74_3(1, acc, acc);
    acc += accum74_3(6);
    acc += guard74_3(acc);
    acc += pick74_3_0(acc, acc + 2);
    S74_4 s4 = mk74_4(acc);
    bump74_4(&s4, 8);
    acc += probe74_4(&s4);
    acc += read74_4(&s4);
    acc += classify74_4(1, acc, acc);
    acc += accum74_4(6);
    acc += guard74_4(acc);
    acc += pick74_4_0(acc, acc + 7);
    acc += pick74_4_1(acc, acc + 9);
    acc += pick74_4_2(acc, acc + 5);
    return clampi(acc);
}
