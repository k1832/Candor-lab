/* GENERATED C mirror of reference module m069. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S69_0;

static S69_0 mk69_0(long a) {
    S69_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe69_0(const S69_0 *s) {
    return s->a + s->n0;
}
static long read69_0(const S69_0 *s) {
    return s->a * 4;
}
static void bump69_0(S69_0 *s, long d) {
    s->a = s->a + d;
}
static long classify69_0(int tag, long a, long b) {
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
static long accum69_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard69_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S69_1;

static S69_1 mk69_1(long a) {
    S69_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe69_1(const S69_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read69_1(const S69_1 *s) {
    return s->a * 3;
}
static void bump69_1(S69_1 *s, long d) {
    s->a = s->a + d;
}
static long classify69_1(int tag, long a, long b) {
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
static long accum69_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard69_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S69_2;

static S69_2 mk69_2(long a) {
    S69_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe69_2(const S69_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read69_2(const S69_2 *s) {
    return s->a * 4;
}
static void bump69_2(S69_2 *s, long d) {
    s->a = s->a + d;
}
static long classify69_2(int tag, long a, long b) {
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
static long accum69_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard69_2(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S69_3;

static S69_3 mk69_3(long a) {
    S69_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe69_3(const S69_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read69_3(const S69_3 *s) {
    return s->a * 3;
}
static void bump69_3(S69_3 *s, long d) {
    s->a = s->a + d;
}
static long classify69_3(int tag, long a, long b) {
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
static long accum69_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard69_3(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S69_4;

static S69_4 mk69_4(long a) {
    S69_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe69_4(const S69_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read69_4(const S69_4 *s) {
    return s->a * 4;
}
static void bump69_4(S69_4 *s, long d) {
    s->a = s->a + d;
}
static long classify69_4(int tag, long a, long b) {
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
static long accum69_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard69_4(long x) {
    return x + 9;
}

long f069(long x) {
    long acc = x;
    acc += f001(x + 1);
    acc += f009(x + 2);
    acc += f013(x + 3);
    acc += f028(x + 4);
    S69_0 s0 = mk69_0(acc);
    bump69_0(&s0, 3);
    acc += probe69_0(&s0);
    acc += read69_0(&s0);
    acc += classify69_0(1, acc, acc);
    acc += accum69_0(5);
    acc += guard69_0(acc);
    S69_1 s1 = mk69_1(acc);
    bump69_1(&s1, 9);
    acc += probe69_1(&s1);
    acc += read69_1(&s1);
    acc += classify69_1(1, acc, acc);
    acc += accum69_1(5);
    acc += guard69_1(acc);
    S69_2 s2 = mk69_2(acc);
    bump69_2(&s2, 7);
    acc += probe69_2(&s2);
    acc += read69_2(&s2);
    acc += classify69_2(1, acc, acc);
    acc += accum69_2(4);
    acc += guard69_2(acc);
    S69_3 s3 = mk69_3(acc);
    bump69_3(&s3, 1);
    acc += probe69_3(&s3);
    acc += read69_3(&s3);
    acc += classify69_3(1, acc, acc);
    acc += accum69_3(4);
    acc += guard69_3(acc);
    S69_4 s4 = mk69_4(acc);
    bump69_4(&s4, 2);
    acc += probe69_4(&s4);
    acc += read69_4(&s4);
    acc += classify69_4(1, acc, acc);
    acc += accum69_4(6);
    acc += guard69_4(acc);
    return clampi(acc);
}
