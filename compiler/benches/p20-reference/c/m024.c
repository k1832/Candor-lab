/* GENERATED C mirror of reference module m024. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S24_0;

static S24_0 mk24_0(long a) {
    S24_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe24_0(const S24_0 *s) {
    return s->a + s->n0;
}
static long read24_0(const S24_0 *s) {
    return s->a * 6;
}
static void bump24_0(S24_0 *s, long d) {
    s->a = s->a + d;
}
static long classify24_0(int tag, long a, long b) {
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
static long accum24_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard24_0(long x) {
    return x + 9;
}

static long pick24_0_0(long a, long b) { return a > b ? a : b; }
static long pick24_0_1(long a, long b) { return a > b ? a : b; }
static long pick24_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S24_1;

static S24_1 mk24_1(long a) {
    S24_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe24_1(const S24_1 *s) {
    return s->a + s->n0;
}
static long read24_1(const S24_1 *s) {
    return s->a * 6;
}
static void bump24_1(S24_1 *s, long d) {
    s->a = s->a + d;
}
static long classify24_1(int tag, long a, long b) {
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
static long accum24_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard24_1(long x) {
    return x + 7;
}

static long pick24_1_0(long a, long b) { return a > b ? a : b; }
static long pick24_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S24_2;

static S24_2 mk24_2(long a) {
    S24_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe24_2(const S24_2 *s) {
    return s->a + s->n0;
}
static long read24_2(const S24_2 *s) {
    return s->a * 6;
}
static void bump24_2(S24_2 *s, long d) {
    s->a = s->a + d;
}
static long classify24_2(int tag, long a, long b) {
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
static long accum24_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard24_2(long x) {
    return x + 2;
}

static long pick24_2_0(long a, long b) { return a > b ? a : b; }
static long pick24_2_1(long a, long b) { return a > b ? a : b; }
static long pick24_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S24_3;

static S24_3 mk24_3(long a) {
    S24_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe24_3(const S24_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read24_3(const S24_3 *s) {
    return s->a * 3;
}
static void bump24_3(S24_3 *s, long d) {
    s->a = s->a + d;
}
static long classify24_3(int tag, long a, long b) {
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
static long accum24_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard24_3(long x) {
    return x + 3;
}

static long pick24_3_0(long a, long b) { return a > b ? a : b; }
static long pick24_3_1(long a, long b) { return a > b ? a : b; }
long f024(long x) {
    long acc = x;
    acc += f002(x + 1);
    S24_0 s0 = mk24_0(acc);
    bump24_0(&s0, 1);
    acc += probe24_0(&s0);
    acc += read24_0(&s0);
    acc += classify24_0(1, acc, acc);
    acc += accum24_0(7);
    acc += guard24_0(acc);
    acc += pick24_0_0(acc, acc + 4);
    acc += pick24_0_1(acc, acc + 9);
    acc += pick24_0_2(acc, acc + 3);
    S24_1 s1 = mk24_1(acc);
    bump24_1(&s1, 2);
    acc += probe24_1(&s1);
    acc += read24_1(&s1);
    acc += classify24_1(1, acc, acc);
    acc += accum24_1(9);
    acc += guard24_1(acc);
    acc += pick24_1_0(acc, acc + 6);
    acc += pick24_1_1(acc, acc + 5);
    S24_2 s2 = mk24_2(acc);
    bump24_2(&s2, 2);
    acc += probe24_2(&s2);
    acc += read24_2(&s2);
    acc += classify24_2(1, acc, acc);
    acc += accum24_2(8);
    acc += guard24_2(acc);
    acc += pick24_2_0(acc, acc + 5);
    acc += pick24_2_1(acc, acc + 1);
    acc += pick24_2_2(acc, acc + 6);
    S24_3 s3 = mk24_3(acc);
    bump24_3(&s3, 6);
    acc += probe24_3(&s3);
    acc += read24_3(&s3);
    acc += classify24_3(1, acc, acc);
    acc += accum24_3(4);
    acc += guard24_3(acc);
    acc += pick24_3_0(acc, acc + 7);
    acc += pick24_3_1(acc, acc + 2);
    return clampi(acc);
}
