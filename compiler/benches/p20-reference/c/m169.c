/* GENERATED C mirror of reference module m169. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S169_0;

static S169_0 mk169_0(long a) {
    S169_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe169_0(const S169_0 *s) {
    return s->a + s->n0;
}
static long read169_0(const S169_0 *s) {
    return s->a * 7;
}
static void bump169_0(S169_0 *s, long d) {
    s->a = s->a + d;
}
static long classify169_0(int tag, long a, long b) {
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
static long accum169_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard169_0(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S169_1;

static S169_1 mk169_1(long a) {
    S169_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe169_1(const S169_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read169_1(const S169_1 *s) {
    return s->a * 7;
}
static void bump169_1(S169_1 *s, long d) {
    s->a = s->a + d;
}
static long classify169_1(int tag, long a, long b) {
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
static long accum169_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard169_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S169_2;

static S169_2 mk169_2(long a) {
    S169_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe169_2(const S169_2 *s) {
    return s->a + s->n0;
}
static long read169_2(const S169_2 *s) {
    return s->a * 4;
}
static void bump169_2(S169_2 *s, long d) {
    s->a = s->a + d;
}
static long classify169_2(int tag, long a, long b) {
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
static long accum169_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard169_2(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S169_3;

static S169_3 mk169_3(long a) {
    S169_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe169_3(const S169_3 *s) {
    return s->a + s->n0;
}
static long read169_3(const S169_3 *s) {
    return s->a * 6;
}
static void bump169_3(S169_3 *s, long d) {
    s->a = s->a + d;
}
static long classify169_3(int tag, long a, long b) {
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
static long accum169_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard169_3(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S169_4;

static S169_4 mk169_4(long a) {
    S169_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe169_4(const S169_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read169_4(const S169_4 *s) {
    return s->a * 5;
}
static void bump169_4(S169_4 *s, long d) {
    s->a = s->a + d;
}
static long classify169_4(int tag, long a, long b) {
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
static long accum169_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard169_4(long x) {
    return x + 4;
}

long f169(long x) {
    long acc = x;
    acc += f065(x + 1);
    acc += f074(x + 2);
    acc += f084(x + 3);
    S169_0 s0 = mk169_0(acc);
    bump169_0(&s0, 3);
    acc += probe169_0(&s0);
    acc += read169_0(&s0);
    acc += classify169_0(1, acc, acc);
    acc += accum169_0(8);
    acc += guard169_0(acc);
    S169_1 s1 = mk169_1(acc);
    bump169_1(&s1, 1);
    acc += probe169_1(&s1);
    acc += read169_1(&s1);
    acc += classify169_1(1, acc, acc);
    acc += accum169_1(3);
    acc += guard169_1(acc);
    S169_2 s2 = mk169_2(acc);
    bump169_2(&s2, 9);
    acc += probe169_2(&s2);
    acc += read169_2(&s2);
    acc += classify169_2(1, acc, acc);
    acc += accum169_2(4);
    acc += guard169_2(acc);
    S169_3 s3 = mk169_3(acc);
    bump169_3(&s3, 5);
    acc += probe169_3(&s3);
    acc += read169_3(&s3);
    acc += classify169_3(1, acc, acc);
    acc += accum169_3(5);
    acc += guard169_3(acc);
    S169_4 s4 = mk169_4(acc);
    bump169_4(&s4, 3);
    acc += probe169_4(&s4);
    acc += read169_4(&s4);
    acc += classify169_4(1, acc, acc);
    acc += accum169_4(7);
    acc += guard169_4(acc);
    return clampi(acc);
}
