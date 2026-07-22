/* GENERATED C mirror of reference module m132. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S132_0;

static S132_0 mk132_0(long a) {
    S132_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe132_0(const S132_0 *s) {
    return s->a + s->n0;
}
static long read132_0(const S132_0 *s) {
    return s->a * 7;
}
static void bump132_0(S132_0 *s, long d) {
    s->a = s->a + d;
}
static long classify132_0(int tag, long a, long b) {
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
static long accum132_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard132_0(long x) {
    return x + 6;
}

static long pick132_0_0(long a, long b) { return a > b ? a : b; }
static long pick132_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S132_1;

static S132_1 mk132_1(long a) {
    S132_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe132_1(const S132_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read132_1(const S132_1 *s) {
    return s->a * 5;
}
static void bump132_1(S132_1 *s, long d) {
    s->a = s->a + d;
}
static long classify132_1(int tag, long a, long b) {
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
static long accum132_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard132_1(long x) {
    return x + 8;
}

static long pick132_1_0(long a, long b) { return a > b ? a : b; }
static long pick132_1_1(long a, long b) { return a > b ? a : b; }
static long pick132_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S132_2;

static S132_2 mk132_2(long a) {
    S132_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe132_2(const S132_2 *s) {
    return s->a + s->n0;
}
static long read132_2(const S132_2 *s) {
    return s->a * 3;
}
static void bump132_2(S132_2 *s, long d) {
    s->a = s->a + d;
}
static long classify132_2(int tag, long a, long b) {
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
static long accum132_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard132_2(long x) {
    return x + 1;
}

static long pick132_2_0(long a, long b) { return a > b ? a : b; }
static long pick132_2_1(long a, long b) { return a > b ? a : b; }
static long pick132_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S132_3;

static S132_3 mk132_3(long a) {
    S132_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe132_3(const S132_3 *s) {
    return s->a + s->n0;
}
static long read132_3(const S132_3 *s) {
    return s->a * 6;
}
static void bump132_3(S132_3 *s, long d) {
    s->a = s->a + d;
}
static long classify132_3(int tag, long a, long b) {
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
static long accum132_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard132_3(long x) {
    return x + 8;
}

static long pick132_3_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S132_4;

static S132_4 mk132_4(long a) {
    S132_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe132_4(const S132_4 *s) {
    return s->a + s->n0;
}
static long read132_4(const S132_4 *s) {
    return s->a * 2;
}
static void bump132_4(S132_4 *s, long d) {
    s->a = s->a + d;
}
static long classify132_4(int tag, long a, long b) {
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
static long accum132_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard132_4(long x) {
    return x + 7;
}

static long pick132_4_0(long a, long b) { return a > b ? a : b; }
static long pick132_4_1(long a, long b) { return a > b ? a : b; }
static long pick132_4_2(long a, long b) { return a > b ? a : b; }
long f132(long x) {
    long acc = x;
    acc += f004(x + 1);
    acc += f084(x + 2);
    acc += f094(x + 3);
    acc += f104(x + 4);
    S132_0 s0 = mk132_0(acc);
    bump132_0(&s0, 3);
    acc += probe132_0(&s0);
    acc += read132_0(&s0);
    acc += classify132_0(1, acc, acc);
    acc += accum132_0(5);
    acc += guard132_0(acc);
    acc += pick132_0_0(acc, acc + 6);
    acc += pick132_0_1(acc, acc + 1);
    S132_1 s1 = mk132_1(acc);
    bump132_1(&s1, 4);
    acc += probe132_1(&s1);
    acc += read132_1(&s1);
    acc += classify132_1(1, acc, acc);
    acc += accum132_1(7);
    acc += guard132_1(acc);
    acc += pick132_1_0(acc, acc + 6);
    acc += pick132_1_1(acc, acc + 1);
    acc += pick132_1_2(acc, acc + 5);
    S132_2 s2 = mk132_2(acc);
    bump132_2(&s2, 4);
    acc += probe132_2(&s2);
    acc += read132_2(&s2);
    acc += classify132_2(1, acc, acc);
    acc += accum132_2(3);
    acc += guard132_2(acc);
    acc += pick132_2_0(acc, acc + 5);
    acc += pick132_2_1(acc, acc + 3);
    acc += pick132_2_2(acc, acc + 8);
    S132_3 s3 = mk132_3(acc);
    bump132_3(&s3, 4);
    acc += probe132_3(&s3);
    acc += read132_3(&s3);
    acc += classify132_3(1, acc, acc);
    acc += accum132_3(5);
    acc += guard132_3(acc);
    acc += pick132_3_0(acc, acc + 4);
    S132_4 s4 = mk132_4(acc);
    bump132_4(&s4, 1);
    acc += probe132_4(&s4);
    acc += read132_4(&s4);
    acc += classify132_4(1, acc, acc);
    acc += accum132_4(5);
    acc += guard132_4(acc);
    acc += pick132_4_0(acc, acc + 5);
    acc += pick132_4_1(acc, acc + 3);
    acc += pick132_4_2(acc, acc + 4);
    return clampi(acc);
}
