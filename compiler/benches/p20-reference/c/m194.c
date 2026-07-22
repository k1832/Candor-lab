/* GENERATED C mirror of reference module m194. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S194_0;

static S194_0 mk194_0(long a) {
    S194_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe194_0(const S194_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read194_0(const S194_0 *s) {
    return s->a * 7;
}
static void bump194_0(S194_0 *s, long d) {
    s->a = s->a + d;
}
static long classify194_0(int tag, long a, long b) {
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
static long accum194_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard194_0(long x) {
    return x + 1;
}

static long pick194_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S194_1;

static S194_1 mk194_1(long a) {
    S194_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe194_1(const S194_1 *s) {
    return s->a + s->n0;
}
static long read194_1(const S194_1 *s) {
    return s->a * 3;
}
static void bump194_1(S194_1 *s, long d) {
    s->a = s->a + d;
}
static long classify194_1(int tag, long a, long b) {
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
static long accum194_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard194_1(long x) {
    return x + 5;
}

static long pick194_1_0(long a, long b) { return a > b ? a : b; }
static long pick194_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S194_2;

static S194_2 mk194_2(long a) {
    S194_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe194_2(const S194_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read194_2(const S194_2 *s) {
    return s->a * 2;
}
static void bump194_2(S194_2 *s, long d) {
    s->a = s->a + d;
}
static long classify194_2(int tag, long a, long b) {
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
static long accum194_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard194_2(long x) {
    return x + 7;
}

static long pick194_2_0(long a, long b) { return a > b ? a : b; }
static long pick194_2_1(long a, long b) { return a > b ? a : b; }
static long pick194_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S194_3;

static S194_3 mk194_3(long a) {
    S194_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe194_3(const S194_3 *s) {
    return s->a + s->n0;
}
static long read194_3(const S194_3 *s) {
    return s->a * 5;
}
static void bump194_3(S194_3 *s, long d) {
    s->a = s->a + d;
}
static long classify194_3(int tag, long a, long b) {
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
static long accum194_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard194_3(long x) {
    return x + 5;
}

static long pick194_3_0(long a, long b) { return a > b ? a : b; }
static long pick194_3_1(long a, long b) { return a > b ? a : b; }
static long pick194_3_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S194_4;

static S194_4 mk194_4(long a) {
    S194_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe194_4(const S194_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read194_4(const S194_4 *s) {
    return s->a * 6;
}
static void bump194_4(S194_4 *s, long d) {
    s->a = s->a + d;
}
static long classify194_4(int tag, long a, long b) {
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
static long accum194_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard194_4(long x) {
    return x + 8;
}

static long pick194_4_0(long a, long b) { return a > b ? a : b; }
static long pick194_4_1(long a, long b) { return a > b ? a : b; }
static long pick194_4_2(long a, long b) { return a > b ? a : b; }
long f194(long x) {
    long acc = x;
    acc += f033(x + 1);
    acc += f054(x + 2);
    acc += f174(x + 3);
    S194_0 s0 = mk194_0(acc);
    bump194_0(&s0, 8);
    acc += probe194_0(&s0);
    acc += read194_0(&s0);
    acc += classify194_0(1, acc, acc);
    acc += accum194_0(6);
    acc += guard194_0(acc);
    acc += pick194_0_0(acc, acc + 6);
    S194_1 s1 = mk194_1(acc);
    bump194_1(&s1, 4);
    acc += probe194_1(&s1);
    acc += read194_1(&s1);
    acc += classify194_1(1, acc, acc);
    acc += accum194_1(7);
    acc += guard194_1(acc);
    acc += pick194_1_0(acc, acc + 8);
    acc += pick194_1_1(acc, acc + 8);
    S194_2 s2 = mk194_2(acc);
    bump194_2(&s2, 1);
    acc += probe194_2(&s2);
    acc += read194_2(&s2);
    acc += classify194_2(1, acc, acc);
    acc += accum194_2(9);
    acc += guard194_2(acc);
    acc += pick194_2_0(acc, acc + 8);
    acc += pick194_2_1(acc, acc + 3);
    acc += pick194_2_2(acc, acc + 3);
    S194_3 s3 = mk194_3(acc);
    bump194_3(&s3, 9);
    acc += probe194_3(&s3);
    acc += read194_3(&s3);
    acc += classify194_3(1, acc, acc);
    acc += accum194_3(7);
    acc += guard194_3(acc);
    acc += pick194_3_0(acc, acc + 1);
    acc += pick194_3_1(acc, acc + 4);
    acc += pick194_3_2(acc, acc + 4);
    S194_4 s4 = mk194_4(acc);
    bump194_4(&s4, 7);
    acc += probe194_4(&s4);
    acc += read194_4(&s4);
    acc += classify194_4(1, acc, acc);
    acc += accum194_4(5);
    acc += guard194_4(acc);
    acc += pick194_4_0(acc, acc + 5);
    acc += pick194_4_1(acc, acc + 4);
    acc += pick194_4_2(acc, acc + 8);
    return clampi(acc);
}
