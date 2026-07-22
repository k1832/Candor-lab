/* GENERATED C mirror of reference module m013. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S13_0;

static S13_0 mk13_0(long a) {
    S13_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe13_0(const S13_0 *s) {
    return s->a + s->n0;
}
static long read13_0(const S13_0 *s) {
    return s->a * 6;
}
static void bump13_0(S13_0 *s, long d) {
    s->a = s->a + d;
}
static long classify13_0(int tag, long a, long b) {
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
static long accum13_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard13_0(long x) {
    return x + 2;
}

static long pick13_0_0(long a, long b) { return a > b ? a : b; }
static long pick13_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S13_1;

static S13_1 mk13_1(long a) {
    S13_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe13_1(const S13_1 *s) {
    return s->a + s->n0;
}
static long read13_1(const S13_1 *s) {
    return s->a * 6;
}
static void bump13_1(S13_1 *s, long d) {
    s->a = s->a + d;
}
static long classify13_1(int tag, long a, long b) {
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
static long accum13_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard13_1(long x) {
    return x + 5;
}

static long pick13_1_0(long a, long b) { return a > b ? a : b; }
static long pick13_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S13_2;

static S13_2 mk13_2(long a) {
    S13_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe13_2(const S13_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read13_2(const S13_2 *s) {
    return s->a * 7;
}
static void bump13_2(S13_2 *s, long d) {
    s->a = s->a + d;
}
static long classify13_2(int tag, long a, long b) {
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
static long accum13_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard13_2(long x) {
    return x + 2;
}

static long pick13_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S13_3;

static S13_3 mk13_3(long a) {
    S13_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe13_3(const S13_3 *s) {
    return s->a + s->n0;
}
static long read13_3(const S13_3 *s) {
    return s->a * 6;
}
static void bump13_3(S13_3 *s, long d) {
    s->a = s->a + d;
}
static long classify13_3(int tag, long a, long b) {
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
static long accum13_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard13_3(long x) {
    return x + 5;
}

static long pick13_3_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S13_4;

static S13_4 mk13_4(long a) {
    S13_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe13_4(const S13_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read13_4(const S13_4 *s) {
    return s->a * 2;
}
static void bump13_4(S13_4 *s, long d) {
    s->a = s->a + d;
}
static long classify13_4(int tag, long a, long b) {
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
static long accum13_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard13_4(long x) {
    return x + 8;
}

static long pick13_4_0(long a, long b) { return a > b ? a : b; }
static long pick13_4_1(long a, long b) { return a > b ? a : b; }
static long pick13_4_2(long a, long b) { return a > b ? a : b; }
long f013(long x) {
    long acc = x;
    acc += f000(x + 1);
    acc += f002(x + 2);
    acc += f006(x + 3);
    acc += f007(x + 4);
    S13_0 s0 = mk13_0(acc);
    bump13_0(&s0, 3);
    acc += probe13_0(&s0);
    acc += read13_0(&s0);
    acc += classify13_0(1, acc, acc);
    acc += accum13_0(9);
    acc += guard13_0(acc);
    acc += pick13_0_0(acc, acc + 6);
    acc += pick13_0_1(acc, acc + 6);
    S13_1 s1 = mk13_1(acc);
    bump13_1(&s1, 2);
    acc += probe13_1(&s1);
    acc += read13_1(&s1);
    acc += classify13_1(1, acc, acc);
    acc += accum13_1(3);
    acc += guard13_1(acc);
    acc += pick13_1_0(acc, acc + 7);
    acc += pick13_1_1(acc, acc + 2);
    S13_2 s2 = mk13_2(acc);
    bump13_2(&s2, 6);
    acc += probe13_2(&s2);
    acc += read13_2(&s2);
    acc += classify13_2(1, acc, acc);
    acc += accum13_2(9);
    acc += guard13_2(acc);
    acc += pick13_2_0(acc, acc + 2);
    S13_3 s3 = mk13_3(acc);
    bump13_3(&s3, 4);
    acc += probe13_3(&s3);
    acc += read13_3(&s3);
    acc += classify13_3(1, acc, acc);
    acc += accum13_3(3);
    acc += guard13_3(acc);
    acc += pick13_3_0(acc, acc + 6);
    S13_4 s4 = mk13_4(acc);
    bump13_4(&s4, 8);
    acc += probe13_4(&s4);
    acc += read13_4(&s4);
    acc += classify13_4(1, acc, acc);
    acc += accum13_4(5);
    acc += guard13_4(acc);
    acc += pick13_4_0(acc, acc + 2);
    acc += pick13_4_1(acc, acc + 6);
    acc += pick13_4_2(acc, acc + 9);
    return clampi(acc);
}
