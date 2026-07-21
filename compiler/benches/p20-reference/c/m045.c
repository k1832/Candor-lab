/* GENERATED C mirror of reference module m045. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S45_0;

static S45_0 mk45_0(long a) {
    S45_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe45_0(const S45_0 *s) {
    return s->a + s->n0;
}
static long read45_0(const S45_0 *s) {
    return s->a * 5;
}
static void bump45_0(S45_0 *s, long d) {
    s->a = s->a + d;
}
static long classify45_0(int tag, long a, long b) {
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
static long accum45_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard45_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S45_1;

static S45_1 mk45_1(long a) {
    S45_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe45_1(const S45_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read45_1(const S45_1 *s) {
    return s->a * 4;
}
static void bump45_1(S45_1 *s, long d) {
    s->a = s->a + d;
}
static long classify45_1(int tag, long a, long b) {
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
static long accum45_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard45_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S45_2;

static S45_2 mk45_2(long a) {
    S45_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe45_2(const S45_2 *s) {
    return s->a + s->n0;
}
static long read45_2(const S45_2 *s) {
    return s->a * 3;
}
static void bump45_2(S45_2 *s, long d) {
    s->a = s->a + d;
}
static long classify45_2(int tag, long a, long b) {
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
static long accum45_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard45_2(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S45_3;

static S45_3 mk45_3(long a) {
    S45_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe45_3(const S45_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read45_3(const S45_3 *s) {
    return s->a * 3;
}
static void bump45_3(S45_3 *s, long d) {
    s->a = s->a + d;
}
static long classify45_3(int tag, long a, long b) {
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
static long accum45_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard45_3(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S45_4;

static S45_4 mk45_4(long a) {
    S45_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe45_4(const S45_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read45_4(const S45_4 *s) {
    return s->a * 5;
}
static void bump45_4(S45_4 *s, long d) {
    s->a = s->a + d;
}
static long classify45_4(int tag, long a, long b) {
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
static long accum45_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard45_4(long x) {
    return x + 3;
}

long f045(long x) {
    long acc = x;
    acc += f008(x + 1);
    acc += f014(x + 2);
    acc += f015(x + 3);
    acc += f019(x + 4);
    S45_0 s0 = mk45_0(acc);
    bump45_0(&s0, 6);
    acc += probe45_0(&s0);
    acc += read45_0(&s0);
    acc += classify45_0(1, acc, acc);
    acc += accum45_0(7);
    acc += guard45_0(acc);
    S45_1 s1 = mk45_1(acc);
    bump45_1(&s1, 6);
    acc += probe45_1(&s1);
    acc += read45_1(&s1);
    acc += classify45_1(1, acc, acc);
    acc += accum45_1(4);
    acc += guard45_1(acc);
    S45_2 s2 = mk45_2(acc);
    bump45_2(&s2, 4);
    acc += probe45_2(&s2);
    acc += read45_2(&s2);
    acc += classify45_2(1, acc, acc);
    acc += accum45_2(3);
    acc += guard45_2(acc);
    S45_3 s3 = mk45_3(acc);
    bump45_3(&s3, 4);
    acc += probe45_3(&s3);
    acc += read45_3(&s3);
    acc += classify45_3(1, acc, acc);
    acc += accum45_3(6);
    acc += guard45_3(acc);
    S45_4 s4 = mk45_4(acc);
    bump45_4(&s4, 9);
    acc += probe45_4(&s4);
    acc += read45_4(&s4);
    acc += classify45_4(1, acc, acc);
    acc += accum45_4(5);
    acc += guard45_4(acc);
    return clampi(acc);
}
