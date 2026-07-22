/* GENERATED C mirror of reference module m188. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S188_0;

static S188_0 mk188_0(long a) {
    S188_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe188_0(const S188_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read188_0(const S188_0 *s) {
    return s->a * 6;
}
static void bump188_0(S188_0 *s, long d) {
    s->a = s->a + d;
}
static long classify188_0(int tag, long a, long b) {
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
static long accum188_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard188_0(long x) {
    return x + 8;
}

static long pick188_0_0(long a, long b) { return a > b ? a : b; }
static long pick188_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S188_1;

static S188_1 mk188_1(long a) {
    S188_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe188_1(const S188_1 *s) {
    return s->a + s->n0;
}
static long read188_1(const S188_1 *s) {
    return s->a * 6;
}
static void bump188_1(S188_1 *s, long d) {
    s->a = s->a + d;
}
static long classify188_1(int tag, long a, long b) {
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
static long accum188_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard188_1(long x) {
    return x + 5;
}

static long pick188_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S188_2;

static S188_2 mk188_2(long a) {
    S188_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe188_2(const S188_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read188_2(const S188_2 *s) {
    return s->a * 6;
}
static void bump188_2(S188_2 *s, long d) {
    s->a = s->a + d;
}
static long classify188_2(int tag, long a, long b) {
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
static long accum188_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard188_2(long x) {
    return x + 9;
}

static long pick188_2_0(long a, long b) { return a > b ? a : b; }
static long pick188_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S188_3;

static S188_3 mk188_3(long a) {
    S188_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe188_3(const S188_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read188_3(const S188_3 *s) {
    return s->a * 3;
}
static void bump188_3(S188_3 *s, long d) {
    s->a = s->a + d;
}
static long classify188_3(int tag, long a, long b) {
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
static long accum188_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard188_3(long x) {
    return x + 7;
}

static long pick188_3_0(long a, long b) { return a > b ? a : b; }
static long pick188_3_1(long a, long b) { return a > b ? a : b; }
static long pick188_3_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S188_4;

static S188_4 mk188_4(long a) {
    S188_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe188_4(const S188_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read188_4(const S188_4 *s) {
    return s->a * 5;
}
static void bump188_4(S188_4 *s, long d) {
    s->a = s->a + d;
}
static long classify188_4(int tag, long a, long b) {
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
static long accum188_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard188_4(long x) {
    return x + 5;
}

static long pick188_4_0(long a, long b) { return a > b ? a : b; }
static long pick188_4_1(long a, long b) { return a > b ? a : b; }
static long pick188_4_2(long a, long b) { return a > b ? a : b; }
long f188(long x) {
    long acc = x;
    acc += f000(x + 1);
    acc += f042(x + 2);
    acc += f078(x + 3);
    S188_0 s0 = mk188_0(acc);
    bump188_0(&s0, 3);
    acc += probe188_0(&s0);
    acc += read188_0(&s0);
    acc += classify188_0(1, acc, acc);
    acc += accum188_0(8);
    acc += guard188_0(acc);
    acc += pick188_0_0(acc, acc + 1);
    acc += pick188_0_1(acc, acc + 2);
    S188_1 s1 = mk188_1(acc);
    bump188_1(&s1, 4);
    acc += probe188_1(&s1);
    acc += read188_1(&s1);
    acc += classify188_1(1, acc, acc);
    acc += accum188_1(5);
    acc += guard188_1(acc);
    acc += pick188_1_0(acc, acc + 7);
    S188_2 s2 = mk188_2(acc);
    bump188_2(&s2, 7);
    acc += probe188_2(&s2);
    acc += read188_2(&s2);
    acc += classify188_2(1, acc, acc);
    acc += accum188_2(7);
    acc += guard188_2(acc);
    acc += pick188_2_0(acc, acc + 4);
    acc += pick188_2_1(acc, acc + 2);
    S188_3 s3 = mk188_3(acc);
    bump188_3(&s3, 3);
    acc += probe188_3(&s3);
    acc += read188_3(&s3);
    acc += classify188_3(1, acc, acc);
    acc += accum188_3(3);
    acc += guard188_3(acc);
    acc += pick188_3_0(acc, acc + 5);
    acc += pick188_3_1(acc, acc + 7);
    acc += pick188_3_2(acc, acc + 9);
    S188_4 s4 = mk188_4(acc);
    bump188_4(&s4, 9);
    acc += probe188_4(&s4);
    acc += read188_4(&s4);
    acc += classify188_4(1, acc, acc);
    acc += accum188_4(4);
    acc += guard188_4(acc);
    acc += pick188_4_0(acc, acc + 2);
    acc += pick188_4_1(acc, acc + 9);
    acc += pick188_4_2(acc, acc + 3);
    return clampi(acc);
}
