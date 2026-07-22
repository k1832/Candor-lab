/* GENERATED C mirror of reference module m138. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S138_0;

static S138_0 mk138_0(long a) {
    S138_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe138_0(const S138_0 *s) {
    return s->a + s->n0;
}
static long read138_0(const S138_0 *s) {
    return s->a * 3;
}
static void bump138_0(S138_0 *s, long d) {
    s->a = s->a + d;
}
static long classify138_0(int tag, long a, long b) {
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
static long accum138_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard138_0(long x) {
    return x + 1;
}

static long pick138_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S138_1;

static S138_1 mk138_1(long a) {
    S138_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe138_1(const S138_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read138_1(const S138_1 *s) {
    return s->a * 5;
}
static void bump138_1(S138_1 *s, long d) {
    s->a = s->a + d;
}
static long classify138_1(int tag, long a, long b) {
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
static long accum138_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard138_1(long x) {
    return x + 8;
}

static long pick138_1_0(long a, long b) { return a > b ? a : b; }
static long pick138_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S138_2;

static S138_2 mk138_2(long a) {
    S138_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe138_2(const S138_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read138_2(const S138_2 *s) {
    return s->a * 4;
}
static void bump138_2(S138_2 *s, long d) {
    s->a = s->a + d;
}
static long classify138_2(int tag, long a, long b) {
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
static long accum138_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard138_2(long x) {
    return x + 4;
}

static long pick138_2_0(long a, long b) { return a > b ? a : b; }
static long pick138_2_1(long a, long b) { return a > b ? a : b; }
static long pick138_2_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S138_3;

static S138_3 mk138_3(long a) {
    S138_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe138_3(const S138_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read138_3(const S138_3 *s) {
    return s->a * 6;
}
static void bump138_3(S138_3 *s, long d) {
    s->a = s->a + d;
}
static long classify138_3(int tag, long a, long b) {
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
static long accum138_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard138_3(long x) {
    return x + 8;
}

static long pick138_3_0(long a, long b) { return a > b ? a : b; }
static long pick138_3_1(long a, long b) { return a > b ? a : b; }
static long pick138_3_2(long a, long b) { return a > b ? a : b; }
long f138(long x) {
    long acc = x;
    acc += f031(x + 1);
    acc += f053(x + 2);
    acc += f107(x + 3);
    S138_0 s0 = mk138_0(acc);
    bump138_0(&s0, 8);
    acc += probe138_0(&s0);
    acc += read138_0(&s0);
    acc += classify138_0(1, acc, acc);
    acc += accum138_0(6);
    acc += guard138_0(acc);
    acc += pick138_0_0(acc, acc + 7);
    S138_1 s1 = mk138_1(acc);
    bump138_1(&s1, 4);
    acc += probe138_1(&s1);
    acc += read138_1(&s1);
    acc += classify138_1(1, acc, acc);
    acc += accum138_1(9);
    acc += guard138_1(acc);
    acc += pick138_1_0(acc, acc + 6);
    acc += pick138_1_1(acc, acc + 3);
    S138_2 s2 = mk138_2(acc);
    bump138_2(&s2, 9);
    acc += probe138_2(&s2);
    acc += read138_2(&s2);
    acc += classify138_2(1, acc, acc);
    acc += accum138_2(8);
    acc += guard138_2(acc);
    acc += pick138_2_0(acc, acc + 2);
    acc += pick138_2_1(acc, acc + 7);
    acc += pick138_2_2(acc, acc + 6);
    S138_3 s3 = mk138_3(acc);
    bump138_3(&s3, 1);
    acc += probe138_3(&s3);
    acc += read138_3(&s3);
    acc += classify138_3(1, acc, acc);
    acc += accum138_3(7);
    acc += guard138_3(acc);
    acc += pick138_3_0(acc, acc + 2);
    acc += pick138_3_1(acc, acc + 8);
    acc += pick138_3_2(acc, acc + 5);
    return clampi(acc);
}
