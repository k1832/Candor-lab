/* GENERATED C mirror of reference module m001. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S1_0;

static S1_0 mk1_0(long a) {
    S1_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe1_0(const S1_0 *s) {
    return s->a + s->n0;
}
static long read1_0(const S1_0 *s) {
    return s->a * 2;
}
static void bump1_0(S1_0 *s, long d) {
    s->a = s->a + d;
}
static long classify1_0(int tag, long a, long b) {
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
static long accum1_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard1_0(long x) {
    return x + 8;
}

static long pick1_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S1_1;

static S1_1 mk1_1(long a) {
    S1_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe1_1(const S1_1 *s) {
    return s->a + s->n0;
}
static long read1_1(const S1_1 *s) {
    return s->a * 5;
}
static void bump1_1(S1_1 *s, long d) {
    s->a = s->a + d;
}
static long classify1_1(int tag, long a, long b) {
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
static long accum1_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard1_1(long x) {
    return x + 9;
}

static long pick1_1_0(long a, long b) { return a > b ? a : b; }
static long pick1_1_1(long a, long b) { return a > b ? a : b; }
static long pick1_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S1_2;

static S1_2 mk1_2(long a) {
    S1_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe1_2(const S1_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read1_2(const S1_2 *s) {
    return s->a * 5;
}
static void bump1_2(S1_2 *s, long d) {
    s->a = s->a + d;
}
static long classify1_2(int tag, long a, long b) {
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
static long accum1_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard1_2(long x) {
    return x + 4;
}

static long pick1_2_0(long a, long b) { return a > b ? a : b; }
static long pick1_2_1(long a, long b) { return a > b ? a : b; }
static long pick1_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S1_3;

static S1_3 mk1_3(long a) {
    S1_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe1_3(const S1_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read1_3(const S1_3 *s) {
    return s->a * 3;
}
static void bump1_3(S1_3 *s, long d) {
    s->a = s->a + d;
}
static long classify1_3(int tag, long a, long b) {
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
static long accum1_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard1_3(long x) {
    return x + 9;
}

static long pick1_3_0(long a, long b) { return a > b ? a : b; }
static long pick1_3_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S1_4;

static S1_4 mk1_4(long a) {
    S1_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe1_4(const S1_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read1_4(const S1_4 *s) {
    return s->a * 7;
}
static void bump1_4(S1_4 *s, long d) {
    s->a = s->a + d;
}
static long classify1_4(int tag, long a, long b) {
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
static long accum1_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard1_4(long x) {
    return x + 2;
}

static long pick1_4_0(long a, long b) { return a > b ? a : b; }
long f001(long x) {
    long acc = x;
    S1_0 s0 = mk1_0(acc);
    bump1_0(&s0, 3);
    acc += probe1_0(&s0);
    acc += read1_0(&s0);
    acc += classify1_0(1, acc, acc);
    acc += accum1_0(8);
    acc += guard1_0(acc);
    acc += pick1_0_0(acc, acc + 3);
    S1_1 s1 = mk1_1(acc);
    bump1_1(&s1, 7);
    acc += probe1_1(&s1);
    acc += read1_1(&s1);
    acc += classify1_1(1, acc, acc);
    acc += accum1_1(6);
    acc += guard1_1(acc);
    acc += pick1_1_0(acc, acc + 4);
    acc += pick1_1_1(acc, acc + 5);
    acc += pick1_1_2(acc, acc + 4);
    S1_2 s2 = mk1_2(acc);
    bump1_2(&s2, 4);
    acc += probe1_2(&s2);
    acc += read1_2(&s2);
    acc += classify1_2(1, acc, acc);
    acc += accum1_2(7);
    acc += guard1_2(acc);
    acc += pick1_2_0(acc, acc + 6);
    acc += pick1_2_1(acc, acc + 5);
    acc += pick1_2_2(acc, acc + 2);
    S1_3 s3 = mk1_3(acc);
    bump1_3(&s3, 4);
    acc += probe1_3(&s3);
    acc += read1_3(&s3);
    acc += classify1_3(1, acc, acc);
    acc += accum1_3(7);
    acc += guard1_3(acc);
    acc += pick1_3_0(acc, acc + 4);
    acc += pick1_3_1(acc, acc + 6);
    S1_4 s4 = mk1_4(acc);
    bump1_4(&s4, 7);
    acc += probe1_4(&s4);
    acc += read1_4(&s4);
    acc += classify1_4(1, acc, acc);
    acc += accum1_4(6);
    acc += guard1_4(acc);
    acc += pick1_4_0(acc, acc + 9);
    return clampi(acc);
}
