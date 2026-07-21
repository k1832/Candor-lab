/* GENERATED C mirror of reference module m052. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S52_0;

static S52_0 mk52_0(long a) {
    S52_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe52_0(const S52_0 *s) {
    return s->a + s->n0;
}
static long read52_0(const S52_0 *s) {
    return s->a * 7;
}
static void bump52_0(S52_0 *s, long d) {
    s->a = s->a + d;
}
static long classify52_0(int tag, long a, long b) {
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
static long accum52_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard52_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S52_1;

static S52_1 mk52_1(long a) {
    S52_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe52_1(const S52_1 *s) {
    return s->a + s->n0;
}
static long read52_1(const S52_1 *s) {
    return s->a * 6;
}
static void bump52_1(S52_1 *s, long d) {
    s->a = s->a + d;
}
static long classify52_1(int tag, long a, long b) {
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
static long accum52_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard52_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S52_2;

static S52_2 mk52_2(long a) {
    S52_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe52_2(const S52_2 *s) {
    return s->a + s->n0;
}
static long read52_2(const S52_2 *s) {
    return s->a * 2;
}
static void bump52_2(S52_2 *s, long d) {
    s->a = s->a + d;
}
static long classify52_2(int tag, long a, long b) {
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
static long accum52_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard52_2(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S52_3;

static S52_3 mk52_3(long a) {
    S52_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe52_3(const S52_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read52_3(const S52_3 *s) {
    return s->a * 7;
}
static void bump52_3(S52_3 *s, long d) {
    s->a = s->a + d;
}
static long classify52_3(int tag, long a, long b) {
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
static long accum52_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard52_3(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S52_4;

static S52_4 mk52_4(long a) {
    S52_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe52_4(const S52_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read52_4(const S52_4 *s) {
    return s->a * 5;
}
static void bump52_4(S52_4 *s, long d) {
    s->a = s->a + d;
}
static long classify52_4(int tag, long a, long b) {
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
static long accum52_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard52_4(long x) {
    return x + 8;
}

long f052(long x) {
    long acc = x;
    acc += f034(x + 1);
    acc += f041(x + 2);
    S52_0 s0 = mk52_0(acc);
    bump52_0(&s0, 8);
    acc += probe52_0(&s0);
    acc += read52_0(&s0);
    acc += classify52_0(1, acc, acc);
    acc += accum52_0(5);
    acc += guard52_0(acc);
    S52_1 s1 = mk52_1(acc);
    bump52_1(&s1, 9);
    acc += probe52_1(&s1);
    acc += read52_1(&s1);
    acc += classify52_1(1, acc, acc);
    acc += accum52_1(8);
    acc += guard52_1(acc);
    S52_2 s2 = mk52_2(acc);
    bump52_2(&s2, 4);
    acc += probe52_2(&s2);
    acc += read52_2(&s2);
    acc += classify52_2(1, acc, acc);
    acc += accum52_2(7);
    acc += guard52_2(acc);
    S52_3 s3 = mk52_3(acc);
    bump52_3(&s3, 7);
    acc += probe52_3(&s3);
    acc += read52_3(&s3);
    acc += classify52_3(1, acc, acc);
    acc += accum52_3(9);
    acc += guard52_3(acc);
    S52_4 s4 = mk52_4(acc);
    bump52_4(&s4, 7);
    acc += probe52_4(&s4);
    acc += read52_4(&s4);
    acc += classify52_4(1, acc, acc);
    acc += accum52_4(6);
    acc += guard52_4(acc);
    return clampi(acc);
}
