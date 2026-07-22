/* GENERATED C mirror of reference module m048. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S48_0;

static S48_0 mk48_0(long a) {
    S48_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe48_0(const S48_0 *s) {
    return s->a + s->n0;
}
static long read48_0(const S48_0 *s) {
    return s->a * 3;
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
        acc += i * 5;
    }
    return acc;
}
static long guard48_0(long x) {
    return x + 6;
}

static long pick48_0_0(long a, long b) { return a > b ? a : b; }
static long pick48_0_1(long a, long b) { return a > b ? a : b; }
static long pick48_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S48_1;

static S48_1 mk48_1(long a) {
    S48_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe48_1(const S48_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read48_1(const S48_1 *s) {
    return s->a * 7;
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
        acc += i * 5;
    }
    return acc;
}
static long guard48_1(long x) {
    return x + 5;
}

static long pick48_1_0(long a, long b) { return a > b ? a : b; }
static long pick48_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S48_2;

static S48_2 mk48_2(long a) {
    S48_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe48_2(const S48_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
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
        acc += i * 5;
    }
    return acc;
}
static long guard48_2(long x) {
    return x + 2;
}

static long pick48_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S48_3;

static S48_3 mk48_3(long a) {
    S48_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe48_3(const S48_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
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
        acc += i * 3;
    }
    return acc;
}
static long guard48_3(long x) {
    return x + 9;
}

static long pick48_3_0(long a, long b) { return a > b ? a : b; }
static long pick48_3_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S48_4;

static S48_4 mk48_4(long a) {
    S48_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe48_4(const S48_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read48_4(const S48_4 *s) {
    return s->a * 6;
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
        acc += i * 2;
    }
    return acc;
}
static long guard48_4(long x) {
    return x + 6;
}

static long pick48_4_0(long a, long b) { return a > b ? a : b; }
static long pick48_4_1(long a, long b) { return a > b ? a : b; }
long f048(long x) {
    long acc = x;
    acc += f024(x + 1);
    acc += f042(x + 2);
    acc += f046(x + 3);
    acc += f047(x + 4);
    S48_0 s0 = mk48_0(acc);
    bump48_0(&s0, 1);
    acc += probe48_0(&s0);
    acc += read48_0(&s0);
    acc += classify48_0(1, acc, acc);
    acc += accum48_0(4);
    acc += guard48_0(acc);
    acc += pick48_0_0(acc, acc + 6);
    acc += pick48_0_1(acc, acc + 6);
    acc += pick48_0_2(acc, acc + 2);
    S48_1 s1 = mk48_1(acc);
    bump48_1(&s1, 8);
    acc += probe48_1(&s1);
    acc += read48_1(&s1);
    acc += classify48_1(1, acc, acc);
    acc += accum48_1(4);
    acc += guard48_1(acc);
    acc += pick48_1_0(acc, acc + 4);
    acc += pick48_1_1(acc, acc + 8);
    S48_2 s2 = mk48_2(acc);
    bump48_2(&s2, 2);
    acc += probe48_2(&s2);
    acc += read48_2(&s2);
    acc += classify48_2(1, acc, acc);
    acc += accum48_2(8);
    acc += guard48_2(acc);
    acc += pick48_2_0(acc, acc + 2);
    S48_3 s3 = mk48_3(acc);
    bump48_3(&s3, 7);
    acc += probe48_3(&s3);
    acc += read48_3(&s3);
    acc += classify48_3(1, acc, acc);
    acc += accum48_3(7);
    acc += guard48_3(acc);
    acc += pick48_3_0(acc, acc + 7);
    acc += pick48_3_1(acc, acc + 6);
    S48_4 s4 = mk48_4(acc);
    bump48_4(&s4, 9);
    acc += probe48_4(&s4);
    acc += read48_4(&s4);
    acc += classify48_4(1, acc, acc);
    acc += accum48_4(9);
    acc += guard48_4(acc);
    acc += pick48_4_0(acc, acc + 3);
    acc += pick48_4_1(acc, acc + 6);
    return clampi(acc);
}
