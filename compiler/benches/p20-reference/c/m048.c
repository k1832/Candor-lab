/* GENERATED C mirror of reference module m048. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S48_0;

static S48_0 mk48_0(long a) {
    S48_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe48_0(const S48_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read48_0(const S48_0 *s) {
    return s->a * 4;
}
static void bump48_0(S48_0 *s, long d) {
    s->a = s->a + d;
}
static long classify48_0(int tag, long a, long b) {
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
static long accum48_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard48_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S48_1;

static S48_1 mk48_1(long a) {
    S48_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe48_1(const S48_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read48_1(const S48_1 *s) {
    return s->a * 6;
}
static void bump48_1(S48_1 *s, long d) {
    s->a = s->a + d;
}
static long classify48_1(int tag, long a, long b) {
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
static long accum48_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard48_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S48_2;

static S48_2 mk48_2(long a) {
    S48_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe48_2(const S48_2 *s) {
    return s->a + s->n0;
}
static long read48_2(const S48_2 *s) {
    return s->a * 3;
}
static void bump48_2(S48_2 *s, long d) {
    s->a = s->a + d;
}
static long classify48_2(int tag, long a, long b) {
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
static long accum48_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard48_2(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S48_3;

static S48_3 mk48_3(long a) {
    S48_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe48_3(const S48_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read48_3(const S48_3 *s) {
    return s->a * 6;
}
static void bump48_3(S48_3 *s, long d) {
    s->a = s->a + d;
}
static long classify48_3(int tag, long a, long b) {
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
static long accum48_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard48_3(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S48_4;

static S48_4 mk48_4(long a) {
    S48_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe48_4(const S48_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read48_4(const S48_4 *s) {
    return s->a * 2;
}
static void bump48_4(S48_4 *s, long d) {
    s->a = s->a + d;
}
static long classify48_4(int tag, long a, long b) {
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
static long accum48_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard48_4(long x) {
    return x + 3;
}

long f048(long x) {
    long acc = x;
    acc += f000(x + 1);
    S48_0 s0 = mk48_0(acc);
    bump48_0(&s0, 8);
    acc += probe48_0(&s0);
    acc += read48_0(&s0);
    acc += classify48_0(1, acc, acc);
    acc += accum48_0(3);
    acc += guard48_0(acc);
    S48_1 s1 = mk48_1(acc);
    bump48_1(&s1, 8);
    acc += probe48_1(&s1);
    acc += read48_1(&s1);
    acc += classify48_1(1, acc, acc);
    acc += accum48_1(9);
    acc += guard48_1(acc);
    S48_2 s2 = mk48_2(acc);
    bump48_2(&s2, 5);
    acc += probe48_2(&s2);
    acc += read48_2(&s2);
    acc += classify48_2(1, acc, acc);
    acc += accum48_2(6);
    acc += guard48_2(acc);
    S48_3 s3 = mk48_3(acc);
    bump48_3(&s3, 8);
    acc += probe48_3(&s3);
    acc += read48_3(&s3);
    acc += classify48_3(1, acc, acc);
    acc += accum48_3(4);
    acc += guard48_3(acc);
    S48_4 s4 = mk48_4(acc);
    bump48_4(&s4, 7);
    acc += probe48_4(&s4);
    acc += read48_4(&s4);
    acc += classify48_4(1, acc, acc);
    acc += accum48_4(3);
    acc += guard48_4(acc);
    return clampi(acc);
}
