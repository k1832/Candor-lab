/* GENERATED C mirror of reference module m165. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S165_0;

static S165_0 mk165_0(long a) {
    S165_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe165_0(const S165_0 *s) {
    return s->a + s->n0;
}
static long read165_0(const S165_0 *s) {
    return s->a * 5;
}
static void bump165_0(S165_0 *s, long d) {
    s->a = s->a + d;
}
static long classify165_0(int tag, long a, long b) {
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
static long accum165_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard165_0(long x) {
    return x + 2;
}

static long pick165_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S165_1;

static S165_1 mk165_1(long a) {
    S165_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe165_1(const S165_1 *s) {
    return s->a + s->n0;
}
static long read165_1(const S165_1 *s) {
    return s->a * 7;
}
static void bump165_1(S165_1 *s, long d) {
    s->a = s->a + d;
}
static long classify165_1(int tag, long a, long b) {
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
static long accum165_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard165_1(long x) {
    return x + 5;
}

static long pick165_1_0(long a, long b) { return a > b ? a : b; }
static long pick165_1_1(long a, long b) { return a > b ? a : b; }
static long pick165_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S165_2;

static S165_2 mk165_2(long a) {
    S165_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe165_2(const S165_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read165_2(const S165_2 *s) {
    return s->a * 4;
}
static void bump165_2(S165_2 *s, long d) {
    s->a = s->a + d;
}
static long classify165_2(int tag, long a, long b) {
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
static long accum165_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard165_2(long x) {
    return x + 5;
}

static long pick165_2_0(long a, long b) { return a > b ? a : b; }
static long pick165_2_1(long a, long b) { return a > b ? a : b; }
long f165(long x) {
    long acc = x;
    acc += f029(x + 1);
    acc += f083(x + 2);
    acc += f132(x + 3);
    S165_0 s0 = mk165_0(acc);
    bump165_0(&s0, 9);
    acc += probe165_0(&s0);
    acc += read165_0(&s0);
    acc += classify165_0(1, acc, acc);
    acc += accum165_0(6);
    acc += guard165_0(acc);
    acc += pick165_0_0(acc, acc + 4);
    S165_1 s1 = mk165_1(acc);
    bump165_1(&s1, 5);
    acc += probe165_1(&s1);
    acc += read165_1(&s1);
    acc += classify165_1(1, acc, acc);
    acc += accum165_1(3);
    acc += guard165_1(acc);
    acc += pick165_1_0(acc, acc + 6);
    acc += pick165_1_1(acc, acc + 9);
    acc += pick165_1_2(acc, acc + 4);
    S165_2 s2 = mk165_2(acc);
    bump165_2(&s2, 5);
    acc += probe165_2(&s2);
    acc += read165_2(&s2);
    acc += classify165_2(1, acc, acc);
    acc += accum165_2(9);
    acc += guard165_2(acc);
    acc += pick165_2_0(acc, acc + 5);
    acc += pick165_2_1(acc, acc + 3);
    return clampi(acc);
}
