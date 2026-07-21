/* GENERATED C mirror of reference module m085. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S85_0;

static S85_0 mk85_0(long a) {
    S85_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe85_0(const S85_0 *s) {
    return s->a + s->n0;
}
static long read85_0(const S85_0 *s) {
    return s->a * 3;
}
static void bump85_0(S85_0 *s, long d) {
    s->a = s->a + d;
}
static long classify85_0(int tag, long a, long b) {
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
static long accum85_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard85_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S85_1;

static S85_1 mk85_1(long a) {
    S85_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe85_1(const S85_1 *s) {
    return s->a + s->n0;
}
static long read85_1(const S85_1 *s) {
    return s->a * 4;
}
static void bump85_1(S85_1 *s, long d) {
    s->a = s->a + d;
}
static long classify85_1(int tag, long a, long b) {
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
static long accum85_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard85_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S85_2;

static S85_2 mk85_2(long a) {
    S85_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe85_2(const S85_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read85_2(const S85_2 *s) {
    return s->a * 6;
}
static void bump85_2(S85_2 *s, long d) {
    s->a = s->a + d;
}
static long classify85_2(int tag, long a, long b) {
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
static long accum85_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard85_2(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S85_3;

static S85_3 mk85_3(long a) {
    S85_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe85_3(const S85_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read85_3(const S85_3 *s) {
    return s->a * 3;
}
static void bump85_3(S85_3 *s, long d) {
    s->a = s->a + d;
}
static long classify85_3(int tag, long a, long b) {
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
static long accum85_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard85_3(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S85_4;

static S85_4 mk85_4(long a) {
    S85_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe85_4(const S85_4 *s) {
    return s->a + s->n0;
}
static long read85_4(const S85_4 *s) {
    return s->a * 2;
}
static void bump85_4(S85_4 *s, long d) {
    s->a = s->a + d;
}
static long classify85_4(int tag, long a, long b) {
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
static long accum85_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard85_4(long x) {
    return x + 1;
}

long f085(long x) {
    long acc = x;
    acc += f003(x + 1);
    acc += f011(x + 2);
    acc += f017(x + 3);
    acc += f055(x + 4);
    S85_0 s0 = mk85_0(acc);
    bump85_0(&s0, 1);
    acc += probe85_0(&s0);
    acc += read85_0(&s0);
    acc += classify85_0(1, acc, acc);
    acc += accum85_0(4);
    acc += guard85_0(acc);
    S85_1 s1 = mk85_1(acc);
    bump85_1(&s1, 6);
    acc += probe85_1(&s1);
    acc += read85_1(&s1);
    acc += classify85_1(1, acc, acc);
    acc += accum85_1(6);
    acc += guard85_1(acc);
    S85_2 s2 = mk85_2(acc);
    bump85_2(&s2, 8);
    acc += probe85_2(&s2);
    acc += read85_2(&s2);
    acc += classify85_2(1, acc, acc);
    acc += accum85_2(3);
    acc += guard85_2(acc);
    S85_3 s3 = mk85_3(acc);
    bump85_3(&s3, 7);
    acc += probe85_3(&s3);
    acc += read85_3(&s3);
    acc += classify85_3(1, acc, acc);
    acc += accum85_3(3);
    acc += guard85_3(acc);
    S85_4 s4 = mk85_4(acc);
    bump85_4(&s4, 8);
    acc += probe85_4(&s4);
    acc += read85_4(&s4);
    acc += classify85_4(1, acc, acc);
    acc += accum85_4(4);
    acc += guard85_4(acc);
    return clampi(acc);
}
