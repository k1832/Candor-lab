/* GENERATED C mirror of reference module m099. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S99_0;

static S99_0 mk99_0(long a) {
    S99_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe99_0(const S99_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read99_0(const S99_0 *s) {
    return s->a * 7;
}
static void bump99_0(S99_0 *s, long d) {
    s->a = s->a + d;
}
static long classify99_0(int tag, long a, long b) {
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
static long accum99_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard99_0(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S99_1;

static S99_1 mk99_1(long a) {
    S99_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe99_1(const S99_1 *s) {
    return s->a + s->n0;
}
static long read99_1(const S99_1 *s) {
    return s->a * 3;
}
static void bump99_1(S99_1 *s, long d) {
    s->a = s->a + d;
}
static long classify99_1(int tag, long a, long b) {
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
static long accum99_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard99_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S99_2;

static S99_2 mk99_2(long a) {
    S99_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe99_2(const S99_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read99_2(const S99_2 *s) {
    return s->a * 3;
}
static void bump99_2(S99_2 *s, long d) {
    s->a = s->a + d;
}
static long classify99_2(int tag, long a, long b) {
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
static long accum99_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard99_2(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S99_3;

static S99_3 mk99_3(long a) {
    S99_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe99_3(const S99_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read99_3(const S99_3 *s) {
    return s->a * 6;
}
static void bump99_3(S99_3 *s, long d) {
    s->a = s->a + d;
}
static long classify99_3(int tag, long a, long b) {
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
static long accum99_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard99_3(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S99_4;

static S99_4 mk99_4(long a) {
    S99_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe99_4(const S99_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read99_4(const S99_4 *s) {
    return s->a * 6;
}
static void bump99_4(S99_4 *s, long d) {
    s->a = s->a + d;
}
static long classify99_4(int tag, long a, long b) {
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
static long accum99_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard99_4(long x) {
    return x + 3;
}

long f099(long x) {
    long acc = x;
    acc += f005(x + 1);
    acc += f037(x + 2);
    S99_0 s0 = mk99_0(acc);
    bump99_0(&s0, 2);
    acc += probe99_0(&s0);
    acc += read99_0(&s0);
    acc += classify99_0(1, acc, acc);
    acc += accum99_0(7);
    acc += guard99_0(acc);
    S99_1 s1 = mk99_1(acc);
    bump99_1(&s1, 8);
    acc += probe99_1(&s1);
    acc += read99_1(&s1);
    acc += classify99_1(1, acc, acc);
    acc += accum99_1(7);
    acc += guard99_1(acc);
    S99_2 s2 = mk99_2(acc);
    bump99_2(&s2, 1);
    acc += probe99_2(&s2);
    acc += read99_2(&s2);
    acc += classify99_2(1, acc, acc);
    acc += accum99_2(9);
    acc += guard99_2(acc);
    S99_3 s3 = mk99_3(acc);
    bump99_3(&s3, 7);
    acc += probe99_3(&s3);
    acc += read99_3(&s3);
    acc += classify99_3(1, acc, acc);
    acc += accum99_3(5);
    acc += guard99_3(acc);
    S99_4 s4 = mk99_4(acc);
    bump99_4(&s4, 9);
    acc += probe99_4(&s4);
    acc += read99_4(&s4);
    acc += classify99_4(1, acc, acc);
    acc += accum99_4(9);
    acc += guard99_4(acc);
    return clampi(acc);
}
