/* GENERATED C mirror of reference module m039. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S39_0;

static S39_0 mk39_0(long a) {
    S39_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe39_0(const S39_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read39_0(const S39_0 *s) {
    return s->a * 6;
}
static void bump39_0(S39_0 *s, long d) {
    s->a = s->a + d;
}
static long classify39_0(int tag, long a, long b) {
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
static long accum39_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard39_0(long x) {
    return x + 7;
}

static long pick39_0_0(long a, long b) { return a > b ? a : b; }
static long pick39_0_1(long a, long b) { return a > b ? a : b; }
static long pick39_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S39_1;

static S39_1 mk39_1(long a) {
    S39_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe39_1(const S39_1 *s) {
    return s->a + s->n0;
}
static long read39_1(const S39_1 *s) {
    return s->a * 2;
}
static void bump39_1(S39_1 *s, long d) {
    s->a = s->a + d;
}
static long classify39_1(int tag, long a, long b) {
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
static long accum39_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard39_1(long x) {
    return x + 7;
}

static long pick39_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S39_2;

static S39_2 mk39_2(long a) {
    S39_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe39_2(const S39_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read39_2(const S39_2 *s) {
    return s->a * 5;
}
static void bump39_2(S39_2 *s, long d) {
    s->a = s->a + d;
}
static long classify39_2(int tag, long a, long b) {
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
static long accum39_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard39_2(long x) {
    return x + 8;
}

static long pick39_2_0(long a, long b) { return a > b ? a : b; }
static long pick39_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S39_3;

static S39_3 mk39_3(long a) {
    S39_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe39_3(const S39_3 *s) {
    return s->a + s->n0;
}
static long read39_3(const S39_3 *s) {
    return s->a * 7;
}
static void bump39_3(S39_3 *s, long d) {
    s->a = s->a + d;
}
static long classify39_3(int tag, long a, long b) {
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
static long accum39_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard39_3(long x) {
    return x + 6;
}

static long pick39_3_0(long a, long b) { return a > b ? a : b; }
static long pick39_3_1(long a, long b) { return a > b ? a : b; }
static long pick39_3_2(long a, long b) { return a > b ? a : b; }
long f039(long x) {
    long acc = x;
    acc += f003(x + 1);
    acc += f014(x + 2);
    S39_0 s0 = mk39_0(acc);
    bump39_0(&s0, 4);
    acc += probe39_0(&s0);
    acc += read39_0(&s0);
    acc += classify39_0(1, acc, acc);
    acc += accum39_0(3);
    acc += guard39_0(acc);
    acc += pick39_0_0(acc, acc + 7);
    acc += pick39_0_1(acc, acc + 5);
    acc += pick39_0_2(acc, acc + 7);
    S39_1 s1 = mk39_1(acc);
    bump39_1(&s1, 7);
    acc += probe39_1(&s1);
    acc += read39_1(&s1);
    acc += classify39_1(1, acc, acc);
    acc += accum39_1(8);
    acc += guard39_1(acc);
    acc += pick39_1_0(acc, acc + 1);
    S39_2 s2 = mk39_2(acc);
    bump39_2(&s2, 2);
    acc += probe39_2(&s2);
    acc += read39_2(&s2);
    acc += classify39_2(1, acc, acc);
    acc += accum39_2(7);
    acc += guard39_2(acc);
    acc += pick39_2_0(acc, acc + 2);
    acc += pick39_2_1(acc, acc + 4);
    S39_3 s3 = mk39_3(acc);
    bump39_3(&s3, 2);
    acc += probe39_3(&s3);
    acc += read39_3(&s3);
    acc += classify39_3(1, acc, acc);
    acc += accum39_3(6);
    acc += guard39_3(acc);
    acc += pick39_3_0(acc, acc + 2);
    acc += pick39_3_1(acc, acc + 8);
    acc += pick39_3_2(acc, acc + 8);
    return clampi(acc);
}
