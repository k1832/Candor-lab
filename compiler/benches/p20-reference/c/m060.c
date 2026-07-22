/* GENERATED C mirror of reference module m060. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S60_0;

static S60_0 mk60_0(long a) {
    S60_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe60_0(const S60_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read60_0(const S60_0 *s) {
    return s->a * 4;
}
static void bump60_0(S60_0 *s, long d) {
    s->a = s->a + d;
}
static long classify60_0(int tag, long a, long b) {
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
static long accum60_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard60_0(long x) {
    return x + 2;
}

static long pick60_0_0(long a, long b) { return a > b ? a : b; }
static long pick60_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S60_1;

static S60_1 mk60_1(long a) {
    S60_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe60_1(const S60_1 *s) {
    return s->a + s->n0;
}
static long read60_1(const S60_1 *s) {
    return s->a * 5;
}
static void bump60_1(S60_1 *s, long d) {
    s->a = s->a + d;
}
static long classify60_1(int tag, long a, long b) {
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
static long accum60_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard60_1(long x) {
    return x + 2;
}

static long pick60_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S60_2;

static S60_2 mk60_2(long a) {
    S60_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe60_2(const S60_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read60_2(const S60_2 *s) {
    return s->a * 5;
}
static void bump60_2(S60_2 *s, long d) {
    s->a = s->a + d;
}
static long classify60_2(int tag, long a, long b) {
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
static long accum60_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard60_2(long x) {
    return x + 8;
}

static long pick60_2_0(long a, long b) { return a > b ? a : b; }
static long pick60_2_1(long a, long b) { return a > b ? a : b; }
long f060(long x) {
    long acc = x;
    acc += f007(x + 1);
    acc += f037(x + 2);
    acc += f040(x + 3);
    acc += f041(x + 4);
    S60_0 s0 = mk60_0(acc);
    bump60_0(&s0, 5);
    acc += probe60_0(&s0);
    acc += read60_0(&s0);
    acc += classify60_0(1, acc, acc);
    acc += accum60_0(7);
    acc += guard60_0(acc);
    acc += pick60_0_0(acc, acc + 1);
    acc += pick60_0_1(acc, acc + 3);
    S60_1 s1 = mk60_1(acc);
    bump60_1(&s1, 6);
    acc += probe60_1(&s1);
    acc += read60_1(&s1);
    acc += classify60_1(1, acc, acc);
    acc += accum60_1(8);
    acc += guard60_1(acc);
    acc += pick60_1_0(acc, acc + 6);
    S60_2 s2 = mk60_2(acc);
    bump60_2(&s2, 3);
    acc += probe60_2(&s2);
    acc += read60_2(&s2);
    acc += classify60_2(1, acc, acc);
    acc += accum60_2(5);
    acc += guard60_2(acc);
    acc += pick60_2_0(acc, acc + 7);
    acc += pick60_2_1(acc, acc + 9);
    return clampi(acc);
}
