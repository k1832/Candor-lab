/* GENERATED C mirror of reference module m174. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S174_0;

static S174_0 mk174_0(long a) {
    S174_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe174_0(const S174_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read174_0(const S174_0 *s) {
    return s->a * 3;
}
static void bump174_0(S174_0 *s, long d) {
    s->a = s->a + d;
}
static long classify174_0(int tag, long a, long b) {
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
static long accum174_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard174_0(long x) {
    return x + 9;
}

static long pick174_0_0(long a, long b) { return a > b ? a : b; }
static long pick174_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S174_1;

static S174_1 mk174_1(long a) {
    S174_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe174_1(const S174_1 *s) {
    return s->a + s->n0;
}
static long read174_1(const S174_1 *s) {
    return s->a * 5;
}
static void bump174_1(S174_1 *s, long d) {
    s->a = s->a + d;
}
static long classify174_1(int tag, long a, long b) {
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
static long accum174_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard174_1(long x) {
    return x + 5;
}

static long pick174_1_0(long a, long b) { return a > b ? a : b; }
static long pick174_1_1(long a, long b) { return a > b ? a : b; }
static long pick174_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S174_2;

static S174_2 mk174_2(long a) {
    S174_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe174_2(const S174_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read174_2(const S174_2 *s) {
    return s->a * 6;
}
static void bump174_2(S174_2 *s, long d) {
    s->a = s->a + d;
}
static long classify174_2(int tag, long a, long b) {
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
static long accum174_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard174_2(long x) {
    return x + 3;
}

static long pick174_2_0(long a, long b) { return a > b ? a : b; }
static long pick174_2_1(long a, long b) { return a > b ? a : b; }
static long pick174_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S174_3;

static S174_3 mk174_3(long a) {
    S174_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe174_3(const S174_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read174_3(const S174_3 *s) {
    return s->a * 7;
}
static void bump174_3(S174_3 *s, long d) {
    s->a = s->a + d;
}
static long classify174_3(int tag, long a, long b) {
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
static long accum174_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard174_3(long x) {
    return x + 1;
}

static long pick174_3_0(long a, long b) { return a > b ? a : b; }
static long pick174_3_1(long a, long b) { return a > b ? a : b; }
long f174(long x) {
    long acc = x;
    acc += f074(x + 1);
    S174_0 s0 = mk174_0(acc);
    bump174_0(&s0, 7);
    acc += probe174_0(&s0);
    acc += read174_0(&s0);
    acc += classify174_0(1, acc, acc);
    acc += accum174_0(5);
    acc += guard174_0(acc);
    acc += pick174_0_0(acc, acc + 8);
    acc += pick174_0_1(acc, acc + 3);
    S174_1 s1 = mk174_1(acc);
    bump174_1(&s1, 3);
    acc += probe174_1(&s1);
    acc += read174_1(&s1);
    acc += classify174_1(1, acc, acc);
    acc += accum174_1(7);
    acc += guard174_1(acc);
    acc += pick174_1_0(acc, acc + 4);
    acc += pick174_1_1(acc, acc + 3);
    acc += pick174_1_2(acc, acc + 5);
    S174_2 s2 = mk174_2(acc);
    bump174_2(&s2, 5);
    acc += probe174_2(&s2);
    acc += read174_2(&s2);
    acc += classify174_2(1, acc, acc);
    acc += accum174_2(7);
    acc += guard174_2(acc);
    acc += pick174_2_0(acc, acc + 2);
    acc += pick174_2_1(acc, acc + 6);
    acc += pick174_2_2(acc, acc + 1);
    S174_3 s3 = mk174_3(acc);
    bump174_3(&s3, 9);
    acc += probe174_3(&s3);
    acc += read174_3(&s3);
    acc += classify174_3(1, acc, acc);
    acc += accum174_3(8);
    acc += guard174_3(acc);
    acc += pick174_3_0(acc, acc + 4);
    acc += pick174_3_1(acc, acc + 7);
    return clampi(acc);
}
