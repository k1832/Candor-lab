/* GENERATED C mirror of reference module m136. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S136_0;

static S136_0 mk136_0(long a) {
    S136_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe136_0(const S136_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read136_0(const S136_0 *s) {
    return s->a * 3;
}
static void bump136_0(S136_0 *s, long d) {
    s->a = s->a + d;
}
static long classify136_0(int tag, long a, long b) {
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
static long accum136_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard136_0(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S136_1;

static S136_1 mk136_1(long a) {
    S136_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe136_1(const S136_1 *s) {
    return s->a + s->n0;
}
static long read136_1(const S136_1 *s) {
    return s->a * 5;
}
static void bump136_1(S136_1 *s, long d) {
    s->a = s->a + d;
}
static long classify136_1(int tag, long a, long b) {
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
static long accum136_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard136_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S136_2;

static S136_2 mk136_2(long a) {
    S136_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe136_2(const S136_2 *s) {
    return s->a + s->n0;
}
static long read136_2(const S136_2 *s) {
    return s->a * 5;
}
static void bump136_2(S136_2 *s, long d) {
    s->a = s->a + d;
}
static long classify136_2(int tag, long a, long b) {
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
static long accum136_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard136_2(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S136_3;

static S136_3 mk136_3(long a) {
    S136_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe136_3(const S136_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read136_3(const S136_3 *s) {
    return s->a * 7;
}
static void bump136_3(S136_3 *s, long d) {
    s->a = s->a + d;
}
static long classify136_3(int tag, long a, long b) {
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
static long accum136_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard136_3(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S136_4;

static S136_4 mk136_4(long a) {
    S136_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe136_4(const S136_4 *s) {
    return s->a + s->n0;
}
static long read136_4(const S136_4 *s) {
    return s->a * 7;
}
static void bump136_4(S136_4 *s, long d) {
    s->a = s->a + d;
}
static long classify136_4(int tag, long a, long b) {
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
static long accum136_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard136_4(long x) {
    return x + 2;
}

long f136(long x) {
    long acc = x;
    acc += f006(x + 1);
    acc += f079(x + 2);
    acc += f106(x + 3);
    S136_0 s0 = mk136_0(acc);
    bump136_0(&s0, 8);
    acc += probe136_0(&s0);
    acc += read136_0(&s0);
    acc += classify136_0(1, acc, acc);
    acc += accum136_0(4);
    acc += guard136_0(acc);
    S136_1 s1 = mk136_1(acc);
    bump136_1(&s1, 3);
    acc += probe136_1(&s1);
    acc += read136_1(&s1);
    acc += classify136_1(1, acc, acc);
    acc += accum136_1(7);
    acc += guard136_1(acc);
    S136_2 s2 = mk136_2(acc);
    bump136_2(&s2, 2);
    acc += probe136_2(&s2);
    acc += read136_2(&s2);
    acc += classify136_2(1, acc, acc);
    acc += accum136_2(4);
    acc += guard136_2(acc);
    S136_3 s3 = mk136_3(acc);
    bump136_3(&s3, 4);
    acc += probe136_3(&s3);
    acc += read136_3(&s3);
    acc += classify136_3(1, acc, acc);
    acc += accum136_3(7);
    acc += guard136_3(acc);
    S136_4 s4 = mk136_4(acc);
    bump136_4(&s4, 9);
    acc += probe136_4(&s4);
    acc += read136_4(&s4);
    acc += classify136_4(1, acc, acc);
    acc += accum136_4(3);
    acc += guard136_4(acc);
    return clampi(acc);
}
