/* GENERATED C mirror of reference module m122. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S122_0;

static S122_0 mk122_0(long a) {
    S122_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe122_0(const S122_0 *s) {
    return s->a + s->n0;
}
static long read122_0(const S122_0 *s) {
    return s->a * 5;
}
static void bump122_0(S122_0 *s, long d) {
    s->a = s->a + d;
}
static long classify122_0(int tag, long a, long b) {
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
static long accum122_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard122_0(long x) {
    return x + 9;
}

static long pick122_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S122_1;

static S122_1 mk122_1(long a) {
    S122_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe122_1(const S122_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read122_1(const S122_1 *s) {
    return s->a * 3;
}
static void bump122_1(S122_1 *s, long d) {
    s->a = s->a + d;
}
static long classify122_1(int tag, long a, long b) {
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
static long accum122_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard122_1(long x) {
    return x + 5;
}

static long pick122_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S122_2;

static S122_2 mk122_2(long a) {
    S122_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe122_2(const S122_2 *s) {
    return s->a + s->n0;
}
static long read122_2(const S122_2 *s) {
    return s->a * 5;
}
static void bump122_2(S122_2 *s, long d) {
    s->a = s->a + d;
}
static long classify122_2(int tag, long a, long b) {
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
static long accum122_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard122_2(long x) {
    return x + 5;
}

static long pick122_2_0(long a, long b) { return a > b ? a : b; }
static long pick122_2_1(long a, long b) { return a > b ? a : b; }
static long pick122_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S122_3;

static S122_3 mk122_3(long a) {
    S122_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe122_3(const S122_3 *s) {
    return s->a + s->n0;
}
static long read122_3(const S122_3 *s) {
    return s->a * 3;
}
static void bump122_3(S122_3 *s, long d) {
    s->a = s->a + d;
}
static long classify122_3(int tag, long a, long b) {
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
static long accum122_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard122_3(long x) {
    return x + 7;
}

static long pick122_3_0(long a, long b) { return a > b ? a : b; }
static long pick122_3_1(long a, long b) { return a > b ? a : b; }
static long pick122_3_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S122_4;

static S122_4 mk122_4(long a) {
    S122_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe122_4(const S122_4 *s) {
    return s->a + s->n0;
}
static long read122_4(const S122_4 *s) {
    return s->a * 4;
}
static void bump122_4(S122_4 *s, long d) {
    s->a = s->a + d;
}
static long classify122_4(int tag, long a, long b) {
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
static long accum122_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard122_4(long x) {
    return x + 6;
}

static long pick122_4_0(long a, long b) { return a > b ? a : b; }
static long pick122_4_1(long a, long b) { return a > b ? a : b; }
long f122(long x) {
    long acc = x;
    acc += f015(x + 1);
    S122_0 s0 = mk122_0(acc);
    bump122_0(&s0, 9);
    acc += probe122_0(&s0);
    acc += read122_0(&s0);
    acc += classify122_0(1, acc, acc);
    acc += accum122_0(9);
    acc += guard122_0(acc);
    acc += pick122_0_0(acc, acc + 2);
    S122_1 s1 = mk122_1(acc);
    bump122_1(&s1, 1);
    acc += probe122_1(&s1);
    acc += read122_1(&s1);
    acc += classify122_1(1, acc, acc);
    acc += accum122_1(5);
    acc += guard122_1(acc);
    acc += pick122_1_0(acc, acc + 5);
    S122_2 s2 = mk122_2(acc);
    bump122_2(&s2, 6);
    acc += probe122_2(&s2);
    acc += read122_2(&s2);
    acc += classify122_2(1, acc, acc);
    acc += accum122_2(8);
    acc += guard122_2(acc);
    acc += pick122_2_0(acc, acc + 1);
    acc += pick122_2_1(acc, acc + 5);
    acc += pick122_2_2(acc, acc + 4);
    S122_3 s3 = mk122_3(acc);
    bump122_3(&s3, 8);
    acc += probe122_3(&s3);
    acc += read122_3(&s3);
    acc += classify122_3(1, acc, acc);
    acc += accum122_3(3);
    acc += guard122_3(acc);
    acc += pick122_3_0(acc, acc + 9);
    acc += pick122_3_1(acc, acc + 1);
    acc += pick122_3_2(acc, acc + 8);
    S122_4 s4 = mk122_4(acc);
    bump122_4(&s4, 6);
    acc += probe122_4(&s4);
    acc += read122_4(&s4);
    acc += classify122_4(1, acc, acc);
    acc += accum122_4(3);
    acc += guard122_4(acc);
    acc += pick122_4_0(acc, acc + 4);
    acc += pick122_4_1(acc, acc + 9);
    return clampi(acc);
}
