/* GENERATED C mirror of reference module m025. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S25_0;

static S25_0 mk25_0(long a) {
    S25_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe25_0(const S25_0 *s) {
    return s->a + s->n0;
}
static long read25_0(const S25_0 *s) {
    return s->a * 6;
}
static void bump25_0(S25_0 *s, long d) {
    s->a = s->a + d;
}
static long classify25_0(int tag, long a, long b) {
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
static long accum25_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard25_0(long x) {
    return x + 8;
}

static long pick25_0_0(long a, long b) { return a > b ? a : b; }
static long pick25_0_1(long a, long b) { return a > b ? a : b; }
static long pick25_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S25_1;

static S25_1 mk25_1(long a) {
    S25_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe25_1(const S25_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read25_1(const S25_1 *s) {
    return s->a * 3;
}
static void bump25_1(S25_1 *s, long d) {
    s->a = s->a + d;
}
static long classify25_1(int tag, long a, long b) {
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
static long accum25_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard25_1(long x) {
    return x + 4;
}

static long pick25_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S25_2;

static S25_2 mk25_2(long a) {
    S25_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe25_2(const S25_2 *s) {
    return s->a + s->n0;
}
static long read25_2(const S25_2 *s) {
    return s->a * 2;
}
static void bump25_2(S25_2 *s, long d) {
    s->a = s->a + d;
}
static long classify25_2(int tag, long a, long b) {
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
static long accum25_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard25_2(long x) {
    return x + 2;
}

static long pick25_2_0(long a, long b) { return a > b ? a : b; }
static long pick25_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S25_3;

static S25_3 mk25_3(long a) {
    S25_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe25_3(const S25_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read25_3(const S25_3 *s) {
    return s->a * 4;
}
static void bump25_3(S25_3 *s, long d) {
    s->a = s->a + d;
}
static long classify25_3(int tag, long a, long b) {
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
static long accum25_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard25_3(long x) {
    return x + 6;
}

static long pick25_3_0(long a, long b) { return a > b ? a : b; }
long f025(long x) {
    long acc = x;
    acc += f008(x + 1);
    acc += f018(x + 2);
    S25_0 s0 = mk25_0(acc);
    bump25_0(&s0, 5);
    acc += probe25_0(&s0);
    acc += read25_0(&s0);
    acc += classify25_0(1, acc, acc);
    acc += accum25_0(8);
    acc += guard25_0(acc);
    acc += pick25_0_0(acc, acc + 1);
    acc += pick25_0_1(acc, acc + 2);
    acc += pick25_0_2(acc, acc + 4);
    S25_1 s1 = mk25_1(acc);
    bump25_1(&s1, 9);
    acc += probe25_1(&s1);
    acc += read25_1(&s1);
    acc += classify25_1(1, acc, acc);
    acc += accum25_1(5);
    acc += guard25_1(acc);
    acc += pick25_1_0(acc, acc + 8);
    S25_2 s2 = mk25_2(acc);
    bump25_2(&s2, 8);
    acc += probe25_2(&s2);
    acc += read25_2(&s2);
    acc += classify25_2(1, acc, acc);
    acc += accum25_2(5);
    acc += guard25_2(acc);
    acc += pick25_2_0(acc, acc + 4);
    acc += pick25_2_1(acc, acc + 9);
    S25_3 s3 = mk25_3(acc);
    bump25_3(&s3, 1);
    acc += probe25_3(&s3);
    acc += read25_3(&s3);
    acc += classify25_3(1, acc, acc);
    acc += accum25_3(7);
    acc += guard25_3(acc);
    acc += pick25_3_0(acc, acc + 4);
    return clampi(acc);
}
