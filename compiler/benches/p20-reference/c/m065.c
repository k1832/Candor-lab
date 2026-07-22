/* GENERATED C mirror of reference module m065. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S65_0;

static S65_0 mk65_0(long a) {
    S65_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe65_0(const S65_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read65_0(const S65_0 *s) {
    return s->a * 4;
}
static void bump65_0(S65_0 *s, long d) {
    s->a = s->a + d;
}
static long classify65_0(int tag, long a, long b) {
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
static long accum65_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard65_0(long x) {
    return x + 1;
}

static long pick65_0_0(long a, long b) { return a > b ? a : b; }
static long pick65_0_1(long a, long b) { return a > b ? a : b; }
static long pick65_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S65_1;

static S65_1 mk65_1(long a) {
    S65_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe65_1(const S65_1 *s) {
    return s->a + s->n0;
}
static long read65_1(const S65_1 *s) {
    return s->a * 6;
}
static void bump65_1(S65_1 *s, long d) {
    s->a = s->a + d;
}
static long classify65_1(int tag, long a, long b) {
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
static long accum65_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard65_1(long x) {
    return x + 7;
}

static long pick65_1_0(long a, long b) { return a > b ? a : b; }
static long pick65_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S65_2;

static S65_2 mk65_2(long a) {
    S65_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe65_2(const S65_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read65_2(const S65_2 *s) {
    return s->a * 3;
}
static void bump65_2(S65_2 *s, long d) {
    s->a = s->a + d;
}
static long classify65_2(int tag, long a, long b) {
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
static long accum65_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard65_2(long x) {
    return x + 9;
}

static long pick65_2_0(long a, long b) { return a > b ? a : b; }
static long pick65_2_1(long a, long b) { return a > b ? a : b; }
static long pick65_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S65_3;

static S65_3 mk65_3(long a) {
    S65_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe65_3(const S65_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read65_3(const S65_3 *s) {
    return s->a * 3;
}
static void bump65_3(S65_3 *s, long d) {
    s->a = s->a + d;
}
static long classify65_3(int tag, long a, long b) {
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
static long accum65_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard65_3(long x) {
    return x + 1;
}

static long pick65_3_0(long a, long b) { return a > b ? a : b; }
static long pick65_3_1(long a, long b) { return a > b ? a : b; }
static long pick65_3_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S65_4;

static S65_4 mk65_4(long a) {
    S65_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe65_4(const S65_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read65_4(const S65_4 *s) {
    return s->a * 5;
}
static void bump65_4(S65_4 *s, long d) {
    s->a = s->a + d;
}
static long classify65_4(int tag, long a, long b) {
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
static long accum65_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard65_4(long x) {
    return x + 1;
}

static long pick65_4_0(long a, long b) { return a > b ? a : b; }
static long pick65_4_1(long a, long b) { return a > b ? a : b; }
static long pick65_4_2(long a, long b) { return a > b ? a : b; }
long f065(long x) {
    long acc = x;
    acc += f017(x + 1);
    acc += f023(x + 2);
    acc += f035(x + 3);
    acc += f042(x + 4);
    S65_0 s0 = mk65_0(acc);
    bump65_0(&s0, 4);
    acc += probe65_0(&s0);
    acc += read65_0(&s0);
    acc += classify65_0(1, acc, acc);
    acc += accum65_0(9);
    acc += guard65_0(acc);
    acc += pick65_0_0(acc, acc + 2);
    acc += pick65_0_1(acc, acc + 2);
    acc += pick65_0_2(acc, acc + 4);
    S65_1 s1 = mk65_1(acc);
    bump65_1(&s1, 7);
    acc += probe65_1(&s1);
    acc += read65_1(&s1);
    acc += classify65_1(1, acc, acc);
    acc += accum65_1(5);
    acc += guard65_1(acc);
    acc += pick65_1_0(acc, acc + 4);
    acc += pick65_1_1(acc, acc + 4);
    S65_2 s2 = mk65_2(acc);
    bump65_2(&s2, 4);
    acc += probe65_2(&s2);
    acc += read65_2(&s2);
    acc += classify65_2(1, acc, acc);
    acc += accum65_2(3);
    acc += guard65_2(acc);
    acc += pick65_2_0(acc, acc + 8);
    acc += pick65_2_1(acc, acc + 9);
    acc += pick65_2_2(acc, acc + 1);
    S65_3 s3 = mk65_3(acc);
    bump65_3(&s3, 3);
    acc += probe65_3(&s3);
    acc += read65_3(&s3);
    acc += classify65_3(1, acc, acc);
    acc += accum65_3(9);
    acc += guard65_3(acc);
    acc += pick65_3_0(acc, acc + 3);
    acc += pick65_3_1(acc, acc + 7);
    acc += pick65_3_2(acc, acc + 2);
    S65_4 s4 = mk65_4(acc);
    bump65_4(&s4, 5);
    acc += probe65_4(&s4);
    acc += read65_4(&s4);
    acc += classify65_4(1, acc, acc);
    acc += accum65_4(7);
    acc += guard65_4(acc);
    acc += pick65_4_0(acc, acc + 3);
    acc += pick65_4_1(acc, acc + 4);
    acc += pick65_4_2(acc, acc + 5);
    return clampi(acc);
}
