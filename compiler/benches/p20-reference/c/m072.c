/* GENERATED C mirror of reference module m072. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S72_0;

static S72_0 mk72_0(long a) {
    S72_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe72_0(const S72_0 *s) {
    return s->a + s->n0;
}
static long read72_0(const S72_0 *s) {
    return s->a * 2;
}
static void bump72_0(S72_0 *s, long d) {
    s->a = s->a + d;
}
static long classify72_0(int tag, long a, long b) {
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
static long accum72_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard72_0(long x) {
    return x + 4;
}

static long pick72_0_0(long a, long b) { return a > b ? a : b; }
static long pick72_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S72_1;

static S72_1 mk72_1(long a) {
    S72_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe72_1(const S72_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read72_1(const S72_1 *s) {
    return s->a * 5;
}
static void bump72_1(S72_1 *s, long d) {
    s->a = s->a + d;
}
static long classify72_1(int tag, long a, long b) {
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
static long accum72_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard72_1(long x) {
    return x + 5;
}

static long pick72_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S72_2;

static S72_2 mk72_2(long a) {
    S72_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe72_2(const S72_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read72_2(const S72_2 *s) {
    return s->a * 3;
}
static void bump72_2(S72_2 *s, long d) {
    s->a = s->a + d;
}
static long classify72_2(int tag, long a, long b) {
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
static long accum72_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard72_2(long x) {
    return x + 6;
}

static long pick72_2_0(long a, long b) { return a > b ? a : b; }
static long pick72_2_1(long a, long b) { return a > b ? a : b; }
static long pick72_2_2(long a, long b) { return a > b ? a : b; }
long f072(long x) {
    long acc = x;
    acc += f038(x + 1);
    acc += f046(x + 2);
    S72_0 s0 = mk72_0(acc);
    bump72_0(&s0, 7);
    acc += probe72_0(&s0);
    acc += read72_0(&s0);
    acc += classify72_0(1, acc, acc);
    acc += accum72_0(5);
    acc += guard72_0(acc);
    acc += pick72_0_0(acc, acc + 8);
    acc += pick72_0_1(acc, acc + 4);
    S72_1 s1 = mk72_1(acc);
    bump72_1(&s1, 6);
    acc += probe72_1(&s1);
    acc += read72_1(&s1);
    acc += classify72_1(1, acc, acc);
    acc += accum72_1(8);
    acc += guard72_1(acc);
    acc += pick72_1_0(acc, acc + 6);
    S72_2 s2 = mk72_2(acc);
    bump72_2(&s2, 6);
    acc += probe72_2(&s2);
    acc += read72_2(&s2);
    acc += classify72_2(1, acc, acc);
    acc += accum72_2(5);
    acc += guard72_2(acc);
    acc += pick72_2_0(acc, acc + 3);
    acc += pick72_2_1(acc, acc + 3);
    acc += pick72_2_2(acc, acc + 9);
    return clampi(acc);
}
