/* GENERATED C mirror of reference module m148. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S148_0;

static S148_0 mk148_0(long a) {
    S148_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe148_0(const S148_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read148_0(const S148_0 *s) {
    return s->a * 7;
}
static void bump148_0(S148_0 *s, long d) {
    s->a = s->a + d;
}
static long classify148_0(int tag, long a, long b) {
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
static long accum148_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard148_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S148_1;

static S148_1 mk148_1(long a) {
    S148_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe148_1(const S148_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read148_1(const S148_1 *s) {
    return s->a * 7;
}
static void bump148_1(S148_1 *s, long d) {
    s->a = s->a + d;
}
static long classify148_1(int tag, long a, long b) {
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
static long accum148_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard148_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S148_2;

static S148_2 mk148_2(long a) {
    S148_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe148_2(const S148_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read148_2(const S148_2 *s) {
    return s->a * 7;
}
static void bump148_2(S148_2 *s, long d) {
    s->a = s->a + d;
}
static long classify148_2(int tag, long a, long b) {
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
static long accum148_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard148_2(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S148_3;

static S148_3 mk148_3(long a) {
    S148_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe148_3(const S148_3 *s) {
    return s->a + s->n0;
}
static long read148_3(const S148_3 *s) {
    return s->a * 6;
}
static void bump148_3(S148_3 *s, long d) {
    s->a = s->a + d;
}
static long classify148_3(int tag, long a, long b) {
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
static long accum148_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard148_3(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S148_4;

static S148_4 mk148_4(long a) {
    S148_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe148_4(const S148_4 *s) {
    return s->a + s->n0;
}
static long read148_4(const S148_4 *s) {
    return s->a * 7;
}
static void bump148_4(S148_4 *s, long d) {
    s->a = s->a + d;
}
static long classify148_4(int tag, long a, long b) {
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
static long accum148_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard148_4(long x) {
    return x + 1;
}

long f148(long x) {
    long acc = x;
    acc += f122(x + 1);
    S148_0 s0 = mk148_0(acc);
    bump148_0(&s0, 2);
    acc += probe148_0(&s0);
    acc += read148_0(&s0);
    acc += classify148_0(1, acc, acc);
    acc += accum148_0(5);
    acc += guard148_0(acc);
    S148_1 s1 = mk148_1(acc);
    bump148_1(&s1, 4);
    acc += probe148_1(&s1);
    acc += read148_1(&s1);
    acc += classify148_1(1, acc, acc);
    acc += accum148_1(7);
    acc += guard148_1(acc);
    S148_2 s2 = mk148_2(acc);
    bump148_2(&s2, 3);
    acc += probe148_2(&s2);
    acc += read148_2(&s2);
    acc += classify148_2(1, acc, acc);
    acc += accum148_2(7);
    acc += guard148_2(acc);
    S148_3 s3 = mk148_3(acc);
    bump148_3(&s3, 4);
    acc += probe148_3(&s3);
    acc += read148_3(&s3);
    acc += classify148_3(1, acc, acc);
    acc += accum148_3(4);
    acc += guard148_3(acc);
    S148_4 s4 = mk148_4(acc);
    bump148_4(&s4, 2);
    acc += probe148_4(&s4);
    acc += read148_4(&s4);
    acc += classify148_4(1, acc, acc);
    acc += accum148_4(7);
    acc += guard148_4(acc);
    return clampi(acc);
}
