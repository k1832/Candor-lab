/* GENERATED C mirror of reference module m066. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S66_0;

static S66_0 mk66_0(long a) {
    S66_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe66_0(const S66_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read66_0(const S66_0 *s) {
    return s->a * 6;
}
static void bump66_0(S66_0 *s, long d) {
    s->a = s->a + d;
}
static long classify66_0(int tag, long a, long b) {
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
static long accum66_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard66_0(long x) {
    return x + 6;
}

static long pick66_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S66_1;

static S66_1 mk66_1(long a) {
    S66_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe66_1(const S66_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read66_1(const S66_1 *s) {
    return s->a * 5;
}
static void bump66_1(S66_1 *s, long d) {
    s->a = s->a + d;
}
static long classify66_1(int tag, long a, long b) {
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
static long accum66_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard66_1(long x) {
    return x + 9;
}

static long pick66_1_0(long a, long b) { return a > b ? a : b; }
static long pick66_1_1(long a, long b) { return a > b ? a : b; }
static long pick66_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S66_2;

static S66_2 mk66_2(long a) {
    S66_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe66_2(const S66_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read66_2(const S66_2 *s) {
    return s->a * 7;
}
static void bump66_2(S66_2 *s, long d) {
    s->a = s->a + d;
}
static long classify66_2(int tag, long a, long b) {
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
static long accum66_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard66_2(long x) {
    return x + 5;
}

static long pick66_2_0(long a, long b) { return a > b ? a : b; }
static long pick66_2_1(long a, long b) { return a > b ? a : b; }
static long pick66_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S66_3;

static S66_3 mk66_3(long a) {
    S66_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe66_3(const S66_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read66_3(const S66_3 *s) {
    return s->a * 5;
}
static void bump66_3(S66_3 *s, long d) {
    s->a = s->a + d;
}
static long classify66_3(int tag, long a, long b) {
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
static long accum66_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard66_3(long x) {
    return x + 1;
}

static long pick66_3_0(long a, long b) { return a > b ? a : b; }
long f066(long x) {
    long acc = x;
    acc += f001(x + 1);
    acc += f018(x + 2);
    acc += f045(x + 3);
    S66_0 s0 = mk66_0(acc);
    bump66_0(&s0, 8);
    acc += probe66_0(&s0);
    acc += read66_0(&s0);
    acc += classify66_0(1, acc, acc);
    acc += accum66_0(4);
    acc += guard66_0(acc);
    acc += pick66_0_0(acc, acc + 2);
    S66_1 s1 = mk66_1(acc);
    bump66_1(&s1, 9);
    acc += probe66_1(&s1);
    acc += read66_1(&s1);
    acc += classify66_1(1, acc, acc);
    acc += accum66_1(4);
    acc += guard66_1(acc);
    acc += pick66_1_0(acc, acc + 6);
    acc += pick66_1_1(acc, acc + 5);
    acc += pick66_1_2(acc, acc + 5);
    S66_2 s2 = mk66_2(acc);
    bump66_2(&s2, 6);
    acc += probe66_2(&s2);
    acc += read66_2(&s2);
    acc += classify66_2(1, acc, acc);
    acc += accum66_2(9);
    acc += guard66_2(acc);
    acc += pick66_2_0(acc, acc + 9);
    acc += pick66_2_1(acc, acc + 1);
    acc += pick66_2_2(acc, acc + 9);
    S66_3 s3 = mk66_3(acc);
    bump66_3(&s3, 1);
    acc += probe66_3(&s3);
    acc += read66_3(&s3);
    acc += classify66_3(1, acc, acc);
    acc += accum66_3(5);
    acc += guard66_3(acc);
    acc += pick66_3_0(acc, acc + 8);
    return clampi(acc);
}
