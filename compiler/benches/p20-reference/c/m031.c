/* GENERATED C mirror of reference module m031. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S31_0;

static S31_0 mk31_0(long a) {
    S31_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe31_0(const S31_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read31_0(const S31_0 *s) {
    return s->a * 3;
}
static void bump31_0(S31_0 *s, long d) {
    s->a = s->a + d;
}
static long classify31_0(int tag, long a, long b) {
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
static long accum31_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard31_0(long x) {
    return x + 7;
}

static long pick31_0_0(long a, long b) { return a > b ? a : b; }
static long pick31_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S31_1;

static S31_1 mk31_1(long a) {
    S31_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe31_1(const S31_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read31_1(const S31_1 *s) {
    return s->a * 2;
}
static void bump31_1(S31_1 *s, long d) {
    s->a = s->a + d;
}
static long classify31_1(int tag, long a, long b) {
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
static long accum31_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard31_1(long x) {
    return x + 4;
}

static long pick31_1_0(long a, long b) { return a > b ? a : b; }
static long pick31_1_1(long a, long b) { return a > b ? a : b; }
static long pick31_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S31_2;

static S31_2 mk31_2(long a) {
    S31_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe31_2(const S31_2 *s) {
    return s->a + s->n0;
}
static long read31_2(const S31_2 *s) {
    return s->a * 5;
}
static void bump31_2(S31_2 *s, long d) {
    s->a = s->a + d;
}
static long classify31_2(int tag, long a, long b) {
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
static long accum31_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard31_2(long x) {
    return x + 8;
}

static long pick31_2_0(long a, long b) { return a > b ? a : b; }
static long pick31_2_1(long a, long b) { return a > b ? a : b; }
static long pick31_2_2(long a, long b) { return a > b ? a : b; }
long f031(long x) {
    long acc = x;
    acc += f000(x + 1);
    acc += f002(x + 2);
    acc += f007(x + 3);
    acc += f015(x + 4);
    S31_0 s0 = mk31_0(acc);
    bump31_0(&s0, 7);
    acc += probe31_0(&s0);
    acc += read31_0(&s0);
    acc += classify31_0(1, acc, acc);
    acc += accum31_0(6);
    acc += guard31_0(acc);
    acc += pick31_0_0(acc, acc + 9);
    acc += pick31_0_1(acc, acc + 6);
    S31_1 s1 = mk31_1(acc);
    bump31_1(&s1, 9);
    acc += probe31_1(&s1);
    acc += read31_1(&s1);
    acc += classify31_1(1, acc, acc);
    acc += accum31_1(7);
    acc += guard31_1(acc);
    acc += pick31_1_0(acc, acc + 1);
    acc += pick31_1_1(acc, acc + 4);
    acc += pick31_1_2(acc, acc + 4);
    S31_2 s2 = mk31_2(acc);
    bump31_2(&s2, 6);
    acc += probe31_2(&s2);
    acc += read31_2(&s2);
    acc += classify31_2(1, acc, acc);
    acc += accum31_2(3);
    acc += guard31_2(acc);
    acc += pick31_2_0(acc, acc + 1);
    acc += pick31_2_1(acc, acc + 8);
    acc += pick31_2_2(acc, acc + 3);
    return clampi(acc);
}
