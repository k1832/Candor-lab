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
    return s->a * 4;
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
        acc += i * 2;
    }
    return acc;
}
static long guard122_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S122_1;

static S122_1 mk122_1(long a) {
    S122_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe122_1(const S122_1 *s) {
    return s->a + s->n0;
}
static long read122_1(const S122_1 *s) {
    return s->a * 2;
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
        acc += i * 5;
    }
    return acc;
}
static long guard122_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S122_2;

static S122_2 mk122_2(long a) {
    S122_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe122_2(const S122_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read122_2(const S122_2 *s) {
    return s->a * 6;
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
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S122_3;

static S122_3 mk122_3(long a) {
    S122_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe122_3(const S122_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read122_3(const S122_3 *s) {
    return s->a * 7;
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
        acc += i * 5;
    }
    return acc;
}
static long guard122_3(long x) {
    return x + 5;
}

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
    return s->a * 5;
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
        acc += i * 5;
    }
    return acc;
}
static long guard122_4(long x) {
    return x + 2;
}

long f122(long x) {
    long acc = x;
    acc += f059(x + 1);
    acc += f078(x + 2);
    S122_0 s0 = mk122_0(acc);
    bump122_0(&s0, 2);
    acc += probe122_0(&s0);
    acc += read122_0(&s0);
    acc += classify122_0(1, acc, acc);
    acc += accum122_0(7);
    acc += guard122_0(acc);
    S122_1 s1 = mk122_1(acc);
    bump122_1(&s1, 4);
    acc += probe122_1(&s1);
    acc += read122_1(&s1);
    acc += classify122_1(1, acc, acc);
    acc += accum122_1(4);
    acc += guard122_1(acc);
    S122_2 s2 = mk122_2(acc);
    bump122_2(&s2, 5);
    acc += probe122_2(&s2);
    acc += read122_2(&s2);
    acc += classify122_2(1, acc, acc);
    acc += accum122_2(5);
    acc += guard122_2(acc);
    S122_3 s3 = mk122_3(acc);
    bump122_3(&s3, 4);
    acc += probe122_3(&s3);
    acc += read122_3(&s3);
    acc += classify122_3(1, acc, acc);
    acc += accum122_3(3);
    acc += guard122_3(acc);
    S122_4 s4 = mk122_4(acc);
    bump122_4(&s4, 6);
    acc += probe122_4(&s4);
    acc += read122_4(&s4);
    acc += classify122_4(1, acc, acc);
    acc += accum122_4(8);
    acc += guard122_4(acc);
    return clampi(acc);
}
