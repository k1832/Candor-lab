/* GENERATED C mirror of reference module m117. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S117_0;

static S117_0 mk117_0(long a) {
    S117_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe117_0(const S117_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read117_0(const S117_0 *s) {
    return s->a * 7;
}
static void bump117_0(S117_0 *s, long d) {
    s->a = s->a + d;
}
static long classify117_0(int tag, long a, long b) {
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
static long accum117_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard117_0(long x) {
    return x + 2;
}

static long pick117_0_0(long a, long b) { return a > b ? a : b; }
static long pick117_0_1(long a, long b) { return a > b ? a : b; }
static long pick117_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S117_1;

static S117_1 mk117_1(long a) {
    S117_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe117_1(const S117_1 *s) {
    return s->a + s->n0;
}
static long read117_1(const S117_1 *s) {
    return s->a * 5;
}
static void bump117_1(S117_1 *s, long d) {
    s->a = s->a + d;
}
static long classify117_1(int tag, long a, long b) {
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
static long accum117_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard117_1(long x) {
    return x + 3;
}

static long pick117_1_0(long a, long b) { return a > b ? a : b; }
static long pick117_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S117_2;

static S117_2 mk117_2(long a) {
    S117_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe117_2(const S117_2 *s) {
    return s->a + s->n0;
}
static long read117_2(const S117_2 *s) {
    return s->a * 7;
}
static void bump117_2(S117_2 *s, long d) {
    s->a = s->a + d;
}
static long classify117_2(int tag, long a, long b) {
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
static long accum117_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard117_2(long x) {
    return x + 8;
}

static long pick117_2_0(long a, long b) { return a > b ? a : b; }
static long pick117_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S117_3;

static S117_3 mk117_3(long a) {
    S117_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe117_3(const S117_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read117_3(const S117_3 *s) {
    return s->a * 5;
}
static void bump117_3(S117_3 *s, long d) {
    s->a = s->a + d;
}
static long classify117_3(int tag, long a, long b) {
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
static long accum117_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard117_3(long x) {
    return x + 1;
}

static long pick117_3_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S117_4;

static S117_4 mk117_4(long a) {
    S117_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe117_4(const S117_4 *s) {
    return s->a + s->n0;
}
static long read117_4(const S117_4 *s) {
    return s->a * 4;
}
static void bump117_4(S117_4 *s, long d) {
    s->a = s->a + d;
}
static long classify117_4(int tag, long a, long b) {
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
static long accum117_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard117_4(long x) {
    return x + 7;
}

static long pick117_4_0(long a, long b) { return a > b ? a : b; }
long f117(long x) {
    long acc = x;
    acc += f025(x + 1);
    acc += f053(x + 2);
    S117_0 s0 = mk117_0(acc);
    bump117_0(&s0, 2);
    acc += probe117_0(&s0);
    acc += read117_0(&s0);
    acc += classify117_0(1, acc, acc);
    acc += accum117_0(3);
    acc += guard117_0(acc);
    acc += pick117_0_0(acc, acc + 1);
    acc += pick117_0_1(acc, acc + 3);
    acc += pick117_0_2(acc, acc + 3);
    S117_1 s1 = mk117_1(acc);
    bump117_1(&s1, 2);
    acc += probe117_1(&s1);
    acc += read117_1(&s1);
    acc += classify117_1(1, acc, acc);
    acc += accum117_1(6);
    acc += guard117_1(acc);
    acc += pick117_1_0(acc, acc + 6);
    acc += pick117_1_1(acc, acc + 6);
    S117_2 s2 = mk117_2(acc);
    bump117_2(&s2, 3);
    acc += probe117_2(&s2);
    acc += read117_2(&s2);
    acc += classify117_2(1, acc, acc);
    acc += accum117_2(6);
    acc += guard117_2(acc);
    acc += pick117_2_0(acc, acc + 5);
    acc += pick117_2_1(acc, acc + 9);
    S117_3 s3 = mk117_3(acc);
    bump117_3(&s3, 3);
    acc += probe117_3(&s3);
    acc += read117_3(&s3);
    acc += classify117_3(1, acc, acc);
    acc += accum117_3(6);
    acc += guard117_3(acc);
    acc += pick117_3_0(acc, acc + 3);
    S117_4 s4 = mk117_4(acc);
    bump117_4(&s4, 9);
    acc += probe117_4(&s4);
    acc += read117_4(&s4);
    acc += classify117_4(1, acc, acc);
    acc += accum117_4(4);
    acc += guard117_4(acc);
    acc += pick117_4_0(acc, acc + 8);
    return clampi(acc);
}
