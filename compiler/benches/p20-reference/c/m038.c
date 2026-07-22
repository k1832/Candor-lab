/* GENERATED C mirror of reference module m038. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S38_0;

static S38_0 mk38_0(long a) {
    S38_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe38_0(const S38_0 *s) {
    return s->a + s->n0;
}
static long read38_0(const S38_0 *s) {
    return s->a * 6;
}
static void bump38_0(S38_0 *s, long d) {
    s->a = s->a + d;
}
static long classify38_0(int tag, long a, long b) {
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
static long accum38_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard38_0(long x) {
    return x + 5;
}

static long pick38_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S38_1;

static S38_1 mk38_1(long a) {
    S38_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe38_1(const S38_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read38_1(const S38_1 *s) {
    return s->a * 6;
}
static void bump38_1(S38_1 *s, long d) {
    s->a = s->a + d;
}
static long classify38_1(int tag, long a, long b) {
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
static long accum38_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard38_1(long x) {
    return x + 9;
}

static long pick38_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S38_2;

static S38_2 mk38_2(long a) {
    S38_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe38_2(const S38_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read38_2(const S38_2 *s) {
    return s->a * 4;
}
static void bump38_2(S38_2 *s, long d) {
    s->a = s->a + d;
}
static long classify38_2(int tag, long a, long b) {
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
static long accum38_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard38_2(long x) {
    return x + 9;
}

static long pick38_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S38_3;

static S38_3 mk38_3(long a) {
    S38_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe38_3(const S38_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read38_3(const S38_3 *s) {
    return s->a * 5;
}
static void bump38_3(S38_3 *s, long d) {
    s->a = s->a + d;
}
static long classify38_3(int tag, long a, long b) {
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
static long accum38_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard38_3(long x) {
    return x + 8;
}

static long pick38_3_0(long a, long b) { return a > b ? a : b; }
static long pick38_3_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S38_4;

static S38_4 mk38_4(long a) {
    S38_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe38_4(const S38_4 *s) {
    return s->a + s->n0;
}
static long read38_4(const S38_4 *s) {
    return s->a * 6;
}
static void bump38_4(S38_4 *s, long d) {
    s->a = s->a + d;
}
static long classify38_4(int tag, long a, long b) {
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
static long accum38_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard38_4(long x) {
    return x + 9;
}

static long pick38_4_0(long a, long b) { return a > b ? a : b; }
static long pick38_4_1(long a, long b) { return a > b ? a : b; }
long f038(long x) {
    long acc = x;
    acc += f000(x + 1);
    acc += f001(x + 2);
    acc += f007(x + 3);
    acc += f013(x + 4);
    S38_0 s0 = mk38_0(acc);
    bump38_0(&s0, 8);
    acc += probe38_0(&s0);
    acc += read38_0(&s0);
    acc += classify38_0(1, acc, acc);
    acc += accum38_0(3);
    acc += guard38_0(acc);
    acc += pick38_0_0(acc, acc + 2);
    S38_1 s1 = mk38_1(acc);
    bump38_1(&s1, 6);
    acc += probe38_1(&s1);
    acc += read38_1(&s1);
    acc += classify38_1(1, acc, acc);
    acc += accum38_1(7);
    acc += guard38_1(acc);
    acc += pick38_1_0(acc, acc + 5);
    S38_2 s2 = mk38_2(acc);
    bump38_2(&s2, 8);
    acc += probe38_2(&s2);
    acc += read38_2(&s2);
    acc += classify38_2(1, acc, acc);
    acc += accum38_2(8);
    acc += guard38_2(acc);
    acc += pick38_2_0(acc, acc + 1);
    S38_3 s3 = mk38_3(acc);
    bump38_3(&s3, 6);
    acc += probe38_3(&s3);
    acc += read38_3(&s3);
    acc += classify38_3(1, acc, acc);
    acc += accum38_3(6);
    acc += guard38_3(acc);
    acc += pick38_3_0(acc, acc + 6);
    acc += pick38_3_1(acc, acc + 6);
    S38_4 s4 = mk38_4(acc);
    bump38_4(&s4, 5);
    acc += probe38_4(&s4);
    acc += read38_4(&s4);
    acc += classify38_4(1, acc, acc);
    acc += accum38_4(8);
    acc += guard38_4(acc);
    acc += pick38_4_0(acc, acc + 2);
    acc += pick38_4_1(acc, acc + 9);
    return clampi(acc);
}
