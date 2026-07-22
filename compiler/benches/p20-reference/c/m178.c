/* GENERATED C mirror of reference module m178. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S178_0;

static S178_0 mk178_0(long a) {
    S178_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe178_0(const S178_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read178_0(const S178_0 *s) {
    return s->a * 6;
}
static void bump178_0(S178_0 *s, long d) {
    s->a = s->a + d;
}
static long classify178_0(int tag, long a, long b) {
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
static long accum178_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard178_0(long x) {
    return x + 5;
}

static long pick178_0_0(long a, long b) { return a > b ? a : b; }
static long pick178_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S178_1;

static S178_1 mk178_1(long a) {
    S178_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe178_1(const S178_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read178_1(const S178_1 *s) {
    return s->a * 4;
}
static void bump178_1(S178_1 *s, long d) {
    s->a = s->a + d;
}
static long classify178_1(int tag, long a, long b) {
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
static long accum178_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard178_1(long x) {
    return x + 9;
}

static long pick178_1_0(long a, long b) { return a > b ? a : b; }
static long pick178_1_1(long a, long b) { return a > b ? a : b; }
static long pick178_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S178_2;

static S178_2 mk178_2(long a) {
    S178_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe178_2(const S178_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read178_2(const S178_2 *s) {
    return s->a * 2;
}
static void bump178_2(S178_2 *s, long d) {
    s->a = s->a + d;
}
static long classify178_2(int tag, long a, long b) {
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
static long accum178_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard178_2(long x) {
    return x + 5;
}

static long pick178_2_0(long a, long b) { return a > b ? a : b; }
static long pick178_2_1(long a, long b) { return a > b ? a : b; }
static long pick178_2_2(long a, long b) { return a > b ? a : b; }
long f178(long x) {
    long acc = x;
    acc += f021(x + 1);
    acc += f077(x + 2);
    acc += f088(x + 3);
    S178_0 s0 = mk178_0(acc);
    bump178_0(&s0, 6);
    acc += probe178_0(&s0);
    acc += read178_0(&s0);
    acc += classify178_0(1, acc, acc);
    acc += accum178_0(6);
    acc += guard178_0(acc);
    acc += pick178_0_0(acc, acc + 8);
    acc += pick178_0_1(acc, acc + 1);
    S178_1 s1 = mk178_1(acc);
    bump178_1(&s1, 1);
    acc += probe178_1(&s1);
    acc += read178_1(&s1);
    acc += classify178_1(1, acc, acc);
    acc += accum178_1(4);
    acc += guard178_1(acc);
    acc += pick178_1_0(acc, acc + 5);
    acc += pick178_1_1(acc, acc + 5);
    acc += pick178_1_2(acc, acc + 5);
    S178_2 s2 = mk178_2(acc);
    bump178_2(&s2, 8);
    acc += probe178_2(&s2);
    acc += read178_2(&s2);
    acc += classify178_2(1, acc, acc);
    acc += accum178_2(4);
    acc += guard178_2(acc);
    acc += pick178_2_0(acc, acc + 9);
    acc += pick178_2_1(acc, acc + 3);
    acc += pick178_2_2(acc, acc + 6);
    return clampi(acc);
}
