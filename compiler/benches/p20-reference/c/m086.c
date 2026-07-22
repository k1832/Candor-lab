/* GENERATED C mirror of reference module m086. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S86_0;

static S86_0 mk86_0(long a) {
    S86_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe86_0(const S86_0 *s) {
    return s->a + s->n0;
}
static long read86_0(const S86_0 *s) {
    return s->a * 4;
}
static void bump86_0(S86_0 *s, long d) {
    s->a = s->a + d;
}
static long classify86_0(int tag, long a, long b) {
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
static long accum86_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard86_0(long x) {
    return x + 3;
}

static long pick86_0_0(long a, long b) { return a > b ? a : b; }
static long pick86_0_1(long a, long b) { return a > b ? a : b; }
static long pick86_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S86_1;

static S86_1 mk86_1(long a) {
    S86_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe86_1(const S86_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read86_1(const S86_1 *s) {
    return s->a * 3;
}
static void bump86_1(S86_1 *s, long d) {
    s->a = s->a + d;
}
static long classify86_1(int tag, long a, long b) {
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
static long accum86_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard86_1(long x) {
    return x + 6;
}

static long pick86_1_0(long a, long b) { return a > b ? a : b; }
static long pick86_1_1(long a, long b) { return a > b ? a : b; }
static long pick86_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S86_2;

static S86_2 mk86_2(long a) {
    S86_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe86_2(const S86_2 *s) {
    return s->a + s->n0;
}
static long read86_2(const S86_2 *s) {
    return s->a * 7;
}
static void bump86_2(S86_2 *s, long d) {
    s->a = s->a + d;
}
static long classify86_2(int tag, long a, long b) {
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
static long accum86_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard86_2(long x) {
    return x + 1;
}

static long pick86_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S86_3;

static S86_3 mk86_3(long a) {
    S86_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe86_3(const S86_3 *s) {
    return s->a + s->n0;
}
static long read86_3(const S86_3 *s) {
    return s->a * 6;
}
static void bump86_3(S86_3 *s, long d) {
    s->a = s->a + d;
}
static long classify86_3(int tag, long a, long b) {
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
static long accum86_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard86_3(long x) {
    return x + 4;
}

static long pick86_3_0(long a, long b) { return a > b ? a : b; }
static long pick86_3_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S86_4;

static S86_4 mk86_4(long a) {
    S86_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe86_4(const S86_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read86_4(const S86_4 *s) {
    return s->a * 6;
}
static void bump86_4(S86_4 *s, long d) {
    s->a = s->a + d;
}
static long classify86_4(int tag, long a, long b) {
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
static long accum86_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard86_4(long x) {
    return x + 9;
}

static long pick86_4_0(long a, long b) { return a > b ? a : b; }
long f086(long x) {
    long acc = x;
    acc += f021(x + 1);
    acc += f058(x + 2);
    S86_0 s0 = mk86_0(acc);
    bump86_0(&s0, 2);
    acc += probe86_0(&s0);
    acc += read86_0(&s0);
    acc += classify86_0(1, acc, acc);
    acc += accum86_0(7);
    acc += guard86_0(acc);
    acc += pick86_0_0(acc, acc + 5);
    acc += pick86_0_1(acc, acc + 1);
    acc += pick86_0_2(acc, acc + 8);
    S86_1 s1 = mk86_1(acc);
    bump86_1(&s1, 5);
    acc += probe86_1(&s1);
    acc += read86_1(&s1);
    acc += classify86_1(1, acc, acc);
    acc += accum86_1(8);
    acc += guard86_1(acc);
    acc += pick86_1_0(acc, acc + 2);
    acc += pick86_1_1(acc, acc + 8);
    acc += pick86_1_2(acc, acc + 3);
    S86_2 s2 = mk86_2(acc);
    bump86_2(&s2, 9);
    acc += probe86_2(&s2);
    acc += read86_2(&s2);
    acc += classify86_2(1, acc, acc);
    acc += accum86_2(8);
    acc += guard86_2(acc);
    acc += pick86_2_0(acc, acc + 5);
    S86_3 s3 = mk86_3(acc);
    bump86_3(&s3, 7);
    acc += probe86_3(&s3);
    acc += read86_3(&s3);
    acc += classify86_3(1, acc, acc);
    acc += accum86_3(9);
    acc += guard86_3(acc);
    acc += pick86_3_0(acc, acc + 5);
    acc += pick86_3_1(acc, acc + 5);
    S86_4 s4 = mk86_4(acc);
    bump86_4(&s4, 6);
    acc += probe86_4(&s4);
    acc += read86_4(&s4);
    acc += classify86_4(1, acc, acc);
    acc += accum86_4(6);
    acc += guard86_4(acc);
    acc += pick86_4_0(acc, acc + 2);
    return clampi(acc);
}
