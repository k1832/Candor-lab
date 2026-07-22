/* GENERATED C mirror of reference module m052. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S52_0;

static S52_0 mk52_0(long a) {
    S52_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe52_0(const S52_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read52_0(const S52_0 *s) {
    return s->a * 7;
}
static void bump52_0(S52_0 *s, long d) {
    s->a = s->a + d;
}
static long classify52_0(int tag, long a, long b) {
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
static long accum52_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard52_0(long x) {
    return x + 7;
}

static long pick52_0_0(long a, long b) { return a > b ? a : b; }
static long pick52_0_1(long a, long b) { return a > b ? a : b; }
static long pick52_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S52_1;

static S52_1 mk52_1(long a) {
    S52_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe52_1(const S52_1 *s) {
    return s->a + s->n0;
}
static long read52_1(const S52_1 *s) {
    return s->a * 5;
}
static void bump52_1(S52_1 *s, long d) {
    s->a = s->a + d;
}
static long classify52_1(int tag, long a, long b) {
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
static long accum52_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard52_1(long x) {
    return x + 2;
}

static long pick52_1_0(long a, long b) { return a > b ? a : b; }
static long pick52_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S52_2;

static S52_2 mk52_2(long a) {
    S52_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe52_2(const S52_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read52_2(const S52_2 *s) {
    return s->a * 2;
}
static void bump52_2(S52_2 *s, long d) {
    s->a = s->a + d;
}
static long classify52_2(int tag, long a, long b) {
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
static long accum52_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard52_2(long x) {
    return x + 9;
}

static long pick52_2_0(long a, long b) { return a > b ? a : b; }
static long pick52_2_1(long a, long b) { return a > b ? a : b; }
static long pick52_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S52_3;

static S52_3 mk52_3(long a) {
    S52_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe52_3(const S52_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read52_3(const S52_3 *s) {
    return s->a * 5;
}
static void bump52_3(S52_3 *s, long d) {
    s->a = s->a + d;
}
static long classify52_3(int tag, long a, long b) {
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
static long accum52_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard52_3(long x) {
    return x + 1;
}

static long pick52_3_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S52_4;

static S52_4 mk52_4(long a) {
    S52_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe52_4(const S52_4 *s) {
    return s->a + s->n0;
}
static long read52_4(const S52_4 *s) {
    return s->a * 5;
}
static void bump52_4(S52_4 *s, long d) {
    s->a = s->a + d;
}
static long classify52_4(int tag, long a, long b) {
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
static long accum52_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard52_4(long x) {
    return x + 5;
}

static long pick52_4_0(long a, long b) { return a > b ? a : b; }
static long pick52_4_1(long a, long b) { return a > b ? a : b; }
static long pick52_4_2(long a, long b) { return a > b ? a : b; }
long f052(long x) {
    long acc = x;
    acc += f004(x + 1);
    S52_0 s0 = mk52_0(acc);
    bump52_0(&s0, 3);
    acc += probe52_0(&s0);
    acc += read52_0(&s0);
    acc += classify52_0(1, acc, acc);
    acc += accum52_0(4);
    acc += guard52_0(acc);
    acc += pick52_0_0(acc, acc + 2);
    acc += pick52_0_1(acc, acc + 5);
    acc += pick52_0_2(acc, acc + 3);
    S52_1 s1 = mk52_1(acc);
    bump52_1(&s1, 7);
    acc += probe52_1(&s1);
    acc += read52_1(&s1);
    acc += classify52_1(1, acc, acc);
    acc += accum52_1(6);
    acc += guard52_1(acc);
    acc += pick52_1_0(acc, acc + 5);
    acc += pick52_1_1(acc, acc + 9);
    S52_2 s2 = mk52_2(acc);
    bump52_2(&s2, 1);
    acc += probe52_2(&s2);
    acc += read52_2(&s2);
    acc += classify52_2(1, acc, acc);
    acc += accum52_2(6);
    acc += guard52_2(acc);
    acc += pick52_2_0(acc, acc + 4);
    acc += pick52_2_1(acc, acc + 1);
    acc += pick52_2_2(acc, acc + 6);
    S52_3 s3 = mk52_3(acc);
    bump52_3(&s3, 4);
    acc += probe52_3(&s3);
    acc += read52_3(&s3);
    acc += classify52_3(1, acc, acc);
    acc += accum52_3(5);
    acc += guard52_3(acc);
    acc += pick52_3_0(acc, acc + 4);
    S52_4 s4 = mk52_4(acc);
    bump52_4(&s4, 8);
    acc += probe52_4(&s4);
    acc += read52_4(&s4);
    acc += classify52_4(1, acc, acc);
    acc += accum52_4(7);
    acc += guard52_4(acc);
    acc += pick52_4_0(acc, acc + 1);
    acc += pick52_4_1(acc, acc + 3);
    acc += pick52_4_2(acc, acc + 4);
    return clampi(acc);
}
