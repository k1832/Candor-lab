/* GENERATED C mirror of reference module m184. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S184_0;

static S184_0 mk184_0(long a) {
    S184_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe184_0(const S184_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read184_0(const S184_0 *s) {
    return s->a * 4;
}
static void bump184_0(S184_0 *s, long d) {
    s->a = s->a + d;
}
static long classify184_0(int tag, long a, long b) {
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
static long accum184_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard184_0(long x) {
    return x + 5;
}

static long pick184_0_0(long a, long b) { return a > b ? a : b; }
static long pick184_0_1(long a, long b) { return a > b ? a : b; }
static long pick184_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S184_1;

static S184_1 mk184_1(long a) {
    S184_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe184_1(const S184_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read184_1(const S184_1 *s) {
    return s->a * 6;
}
static void bump184_1(S184_1 *s, long d) {
    s->a = s->a + d;
}
static long classify184_1(int tag, long a, long b) {
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
static long accum184_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard184_1(long x) {
    return x + 1;
}

static long pick184_1_0(long a, long b) { return a > b ? a : b; }
static long pick184_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S184_2;

static S184_2 mk184_2(long a) {
    S184_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe184_2(const S184_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read184_2(const S184_2 *s) {
    return s->a * 5;
}
static void bump184_2(S184_2 *s, long d) {
    s->a = s->a + d;
}
static long classify184_2(int tag, long a, long b) {
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
static long accum184_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard184_2(long x) {
    return x + 7;
}

static long pick184_2_0(long a, long b) { return a > b ? a : b; }
static long pick184_2_1(long a, long b) { return a > b ? a : b; }
static long pick184_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S184_3;

static S184_3 mk184_3(long a) {
    S184_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe184_3(const S184_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read184_3(const S184_3 *s) {
    return s->a * 3;
}
static void bump184_3(S184_3 *s, long d) {
    s->a = s->a + d;
}
static long classify184_3(int tag, long a, long b) {
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
static long accum184_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard184_3(long x) {
    return x + 9;
}

static long pick184_3_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S184_4;

static S184_4 mk184_4(long a) {
    S184_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe184_4(const S184_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read184_4(const S184_4 *s) {
    return s->a * 6;
}
static void bump184_4(S184_4 *s, long d) {
    s->a = s->a + d;
}
static long classify184_4(int tag, long a, long b) {
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
static long accum184_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard184_4(long x) {
    return x + 4;
}

static long pick184_4_0(long a, long b) { return a > b ? a : b; }
static long pick184_4_1(long a, long b) { return a > b ? a : b; }
long f184(long x) {
    long acc = x;
    acc += f121(x + 1);
    S184_0 s0 = mk184_0(acc);
    bump184_0(&s0, 6);
    acc += probe184_0(&s0);
    acc += read184_0(&s0);
    acc += classify184_0(1, acc, acc);
    acc += accum184_0(9);
    acc += guard184_0(acc);
    acc += pick184_0_0(acc, acc + 5);
    acc += pick184_0_1(acc, acc + 6);
    acc += pick184_0_2(acc, acc + 1);
    S184_1 s1 = mk184_1(acc);
    bump184_1(&s1, 3);
    acc += probe184_1(&s1);
    acc += read184_1(&s1);
    acc += classify184_1(1, acc, acc);
    acc += accum184_1(6);
    acc += guard184_1(acc);
    acc += pick184_1_0(acc, acc + 4);
    acc += pick184_1_1(acc, acc + 7);
    S184_2 s2 = mk184_2(acc);
    bump184_2(&s2, 4);
    acc += probe184_2(&s2);
    acc += read184_2(&s2);
    acc += classify184_2(1, acc, acc);
    acc += accum184_2(7);
    acc += guard184_2(acc);
    acc += pick184_2_0(acc, acc + 5);
    acc += pick184_2_1(acc, acc + 2);
    acc += pick184_2_2(acc, acc + 3);
    S184_3 s3 = mk184_3(acc);
    bump184_3(&s3, 9);
    acc += probe184_3(&s3);
    acc += read184_3(&s3);
    acc += classify184_3(1, acc, acc);
    acc += accum184_3(3);
    acc += guard184_3(acc);
    acc += pick184_3_0(acc, acc + 8);
    S184_4 s4 = mk184_4(acc);
    bump184_4(&s4, 7);
    acc += probe184_4(&s4);
    acc += read184_4(&s4);
    acc += classify184_4(1, acc, acc);
    acc += accum184_4(7);
    acc += guard184_4(acc);
    acc += pick184_4_0(acc, acc + 1);
    acc += pick184_4_1(acc, acc + 4);
    return clampi(acc);
}
