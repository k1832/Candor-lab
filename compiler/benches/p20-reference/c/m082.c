/* GENERATED C mirror of reference module m082. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S82_0;

static S82_0 mk82_0(long a) {
    S82_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe82_0(const S82_0 *s) {
    return s->a + s->n0;
}
static long read82_0(const S82_0 *s) {
    return s->a * 3;
}
static void bump82_0(S82_0 *s, long d) {
    s->a = s->a + d;
}
static long classify82_0(int tag, long a, long b) {
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
static long accum82_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard82_0(long x) {
    return x + 5;
}

static long pick82_0_0(long a, long b) { return a > b ? a : b; }
static long pick82_0_1(long a, long b) { return a > b ? a : b; }
static long pick82_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S82_1;

static S82_1 mk82_1(long a) {
    S82_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe82_1(const S82_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read82_1(const S82_1 *s) {
    return s->a * 5;
}
static void bump82_1(S82_1 *s, long d) {
    s->a = s->a + d;
}
static long classify82_1(int tag, long a, long b) {
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
static long accum82_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard82_1(long x) {
    return x + 3;
}

static long pick82_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S82_2;

static S82_2 mk82_2(long a) {
    S82_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe82_2(const S82_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read82_2(const S82_2 *s) {
    return s->a * 2;
}
static void bump82_2(S82_2 *s, long d) {
    s->a = s->a + d;
}
static long classify82_2(int tag, long a, long b) {
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
static long accum82_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard82_2(long x) {
    return x + 2;
}

static long pick82_2_0(long a, long b) { return a > b ? a : b; }
static long pick82_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S82_3;

static S82_3 mk82_3(long a) {
    S82_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe82_3(const S82_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read82_3(const S82_3 *s) {
    return s->a * 7;
}
static void bump82_3(S82_3 *s, long d) {
    s->a = s->a + d;
}
static long classify82_3(int tag, long a, long b) {
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
static long accum82_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard82_3(long x) {
    return x + 4;
}

static long pick82_3_0(long a, long b) { return a > b ? a : b; }
long f082(long x) {
    long acc = x;
    acc += f053(x + 1);
    S82_0 s0 = mk82_0(acc);
    bump82_0(&s0, 1);
    acc += probe82_0(&s0);
    acc += read82_0(&s0);
    acc += classify82_0(1, acc, acc);
    acc += accum82_0(5);
    acc += guard82_0(acc);
    acc += pick82_0_0(acc, acc + 9);
    acc += pick82_0_1(acc, acc + 2);
    acc += pick82_0_2(acc, acc + 3);
    S82_1 s1 = mk82_1(acc);
    bump82_1(&s1, 2);
    acc += probe82_1(&s1);
    acc += read82_1(&s1);
    acc += classify82_1(1, acc, acc);
    acc += accum82_1(6);
    acc += guard82_1(acc);
    acc += pick82_1_0(acc, acc + 1);
    S82_2 s2 = mk82_2(acc);
    bump82_2(&s2, 8);
    acc += probe82_2(&s2);
    acc += read82_2(&s2);
    acc += classify82_2(1, acc, acc);
    acc += accum82_2(8);
    acc += guard82_2(acc);
    acc += pick82_2_0(acc, acc + 4);
    acc += pick82_2_1(acc, acc + 7);
    S82_3 s3 = mk82_3(acc);
    bump82_3(&s3, 2);
    acc += probe82_3(&s3);
    acc += read82_3(&s3);
    acc += classify82_3(1, acc, acc);
    acc += accum82_3(6);
    acc += guard82_3(acc);
    acc += pick82_3_0(acc, acc + 9);
    return clampi(acc);
}
