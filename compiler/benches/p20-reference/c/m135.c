/* GENERATED C mirror of reference module m135. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S135_0;

static S135_0 mk135_0(long a) {
    S135_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe135_0(const S135_0 *s) {
    return s->a + s->n0;
}
static long read135_0(const S135_0 *s) {
    return s->a * 4;
}
static void bump135_0(S135_0 *s, long d) {
    s->a = s->a + d;
}
static long classify135_0(int tag, long a, long b) {
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
static long accum135_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard135_0(long x) {
    return x + 3;
}

static long pick135_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S135_1;

static S135_1 mk135_1(long a) {
    S135_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe135_1(const S135_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read135_1(const S135_1 *s) {
    return s->a * 3;
}
static void bump135_1(S135_1 *s, long d) {
    s->a = s->a + d;
}
static long classify135_1(int tag, long a, long b) {
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
static long accum135_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard135_1(long x) {
    return x + 7;
}

static long pick135_1_0(long a, long b) { return a > b ? a : b; }
static long pick135_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S135_2;

static S135_2 mk135_2(long a) {
    S135_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe135_2(const S135_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read135_2(const S135_2 *s) {
    return s->a * 6;
}
static void bump135_2(S135_2 *s, long d) {
    s->a = s->a + d;
}
static long classify135_2(int tag, long a, long b) {
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
static long accum135_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard135_2(long x) {
    return x + 6;
}

static long pick135_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S135_3;

static S135_3 mk135_3(long a) {
    S135_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe135_3(const S135_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read135_3(const S135_3 *s) {
    return s->a * 7;
}
static void bump135_3(S135_3 *s, long d) {
    s->a = s->a + d;
}
static long classify135_3(int tag, long a, long b) {
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
static long accum135_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard135_3(long x) {
    return x + 3;
}

static long pick135_3_0(long a, long b) { return a > b ? a : b; }
static long pick135_3_1(long a, long b) { return a > b ? a : b; }
static long pick135_3_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S135_4;

static S135_4 mk135_4(long a) {
    S135_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe135_4(const S135_4 *s) {
    return s->a + s->n0;
}
static long read135_4(const S135_4 *s) {
    return s->a * 2;
}
static void bump135_4(S135_4 *s, long d) {
    s->a = s->a + d;
}
static long classify135_4(int tag, long a, long b) {
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
static long accum135_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard135_4(long x) {
    return x + 2;
}

static long pick135_4_0(long a, long b) { return a > b ? a : b; }
long f135(long x) {
    long acc = x;
    acc += f064(x + 1);
    S135_0 s0 = mk135_0(acc);
    bump135_0(&s0, 5);
    acc += probe135_0(&s0);
    acc += read135_0(&s0);
    acc += classify135_0(1, acc, acc);
    acc += accum135_0(3);
    acc += guard135_0(acc);
    acc += pick135_0_0(acc, acc + 6);
    S135_1 s1 = mk135_1(acc);
    bump135_1(&s1, 6);
    acc += probe135_1(&s1);
    acc += read135_1(&s1);
    acc += classify135_1(1, acc, acc);
    acc += accum135_1(8);
    acc += guard135_1(acc);
    acc += pick135_1_0(acc, acc + 9);
    acc += pick135_1_1(acc, acc + 5);
    S135_2 s2 = mk135_2(acc);
    bump135_2(&s2, 2);
    acc += probe135_2(&s2);
    acc += read135_2(&s2);
    acc += classify135_2(1, acc, acc);
    acc += accum135_2(7);
    acc += guard135_2(acc);
    acc += pick135_2_0(acc, acc + 2);
    S135_3 s3 = mk135_3(acc);
    bump135_3(&s3, 8);
    acc += probe135_3(&s3);
    acc += read135_3(&s3);
    acc += classify135_3(1, acc, acc);
    acc += accum135_3(7);
    acc += guard135_3(acc);
    acc += pick135_3_0(acc, acc + 7);
    acc += pick135_3_1(acc, acc + 4);
    acc += pick135_3_2(acc, acc + 6);
    S135_4 s4 = mk135_4(acc);
    bump135_4(&s4, 4);
    acc += probe135_4(&s4);
    acc += read135_4(&s4);
    acc += classify135_4(1, acc, acc);
    acc += accum135_4(3);
    acc += guard135_4(acc);
    acc += pick135_4_0(acc, acc + 5);
    return clampi(acc);
}
