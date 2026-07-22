/* GENERATED C mirror of reference module m035. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S35_0;

static S35_0 mk35_0(long a) {
    S35_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe35_0(const S35_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read35_0(const S35_0 *s) {
    return s->a * 4;
}
static void bump35_0(S35_0 *s, long d) {
    s->a = s->a + d;
}
static long classify35_0(int tag, long a, long b) {
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
static long accum35_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard35_0(long x) {
    return x + 2;
}

static long pick35_0_0(long a, long b) { return a > b ? a : b; }
static long pick35_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S35_1;

static S35_1 mk35_1(long a) {
    S35_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe35_1(const S35_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read35_1(const S35_1 *s) {
    return s->a * 7;
}
static void bump35_1(S35_1 *s, long d) {
    s->a = s->a + d;
}
static long classify35_1(int tag, long a, long b) {
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
static long accum35_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard35_1(long x) {
    return x + 2;
}

static long pick35_1_0(long a, long b) { return a > b ? a : b; }
static long pick35_1_1(long a, long b) { return a > b ? a : b; }
static long pick35_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S35_2;

static S35_2 mk35_2(long a) {
    S35_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe35_2(const S35_2 *s) {
    return s->a + s->n0;
}
static long read35_2(const S35_2 *s) {
    return s->a * 3;
}
static void bump35_2(S35_2 *s, long d) {
    s->a = s->a + d;
}
static long classify35_2(int tag, long a, long b) {
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
static long accum35_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard35_2(long x) {
    return x + 2;
}

static long pick35_2_0(long a, long b) { return a > b ? a : b; }
static long pick35_2_1(long a, long b) { return a > b ? a : b; }
static long pick35_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S35_3;

static S35_3 mk35_3(long a) {
    S35_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe35_3(const S35_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read35_3(const S35_3 *s) {
    return s->a * 3;
}
static void bump35_3(S35_3 *s, long d) {
    s->a = s->a + d;
}
static long classify35_3(int tag, long a, long b) {
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
static long accum35_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard35_3(long x) {
    return x + 4;
}

static long pick35_3_0(long a, long b) { return a > b ? a : b; }
static long pick35_3_1(long a, long b) { return a > b ? a : b; }
long f035(long x) {
    long acc = x;
    acc += f011(x + 1);
    acc += f015(x + 2);
    acc += f018(x + 3);
    acc += f019(x + 4);
    S35_0 s0 = mk35_0(acc);
    bump35_0(&s0, 2);
    acc += probe35_0(&s0);
    acc += read35_0(&s0);
    acc += classify35_0(1, acc, acc);
    acc += accum35_0(7);
    acc += guard35_0(acc);
    acc += pick35_0_0(acc, acc + 7);
    acc += pick35_0_1(acc, acc + 3);
    S35_1 s1 = mk35_1(acc);
    bump35_1(&s1, 2);
    acc += probe35_1(&s1);
    acc += read35_1(&s1);
    acc += classify35_1(1, acc, acc);
    acc += accum35_1(9);
    acc += guard35_1(acc);
    acc += pick35_1_0(acc, acc + 5);
    acc += pick35_1_1(acc, acc + 8);
    acc += pick35_1_2(acc, acc + 9);
    S35_2 s2 = mk35_2(acc);
    bump35_2(&s2, 7);
    acc += probe35_2(&s2);
    acc += read35_2(&s2);
    acc += classify35_2(1, acc, acc);
    acc += accum35_2(7);
    acc += guard35_2(acc);
    acc += pick35_2_0(acc, acc + 2);
    acc += pick35_2_1(acc, acc + 4);
    acc += pick35_2_2(acc, acc + 2);
    S35_3 s3 = mk35_3(acc);
    bump35_3(&s3, 1);
    acc += probe35_3(&s3);
    acc += read35_3(&s3);
    acc += classify35_3(1, acc, acc);
    acc += accum35_3(8);
    acc += guard35_3(acc);
    acc += pick35_3_0(acc, acc + 1);
    acc += pick35_3_1(acc, acc + 5);
    return clampi(acc);
}
