/* GENERATED C mirror of reference module m142. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S142_0;

static S142_0 mk142_0(long a) {
    S142_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe142_0(const S142_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read142_0(const S142_0 *s) {
    return s->a * 6;
}
static void bump142_0(S142_0 *s, long d) {
    s->a = s->a + d;
}
static long classify142_0(int tag, long a, long b) {
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
static long accum142_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard142_0(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S142_1;

static S142_1 mk142_1(long a) {
    S142_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe142_1(const S142_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read142_1(const S142_1 *s) {
    return s->a * 7;
}
static void bump142_1(S142_1 *s, long d) {
    s->a = s->a + d;
}
static long classify142_1(int tag, long a, long b) {
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
static long accum142_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard142_1(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S142_2;

static S142_2 mk142_2(long a) {
    S142_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe142_2(const S142_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read142_2(const S142_2 *s) {
    return s->a * 6;
}
static void bump142_2(S142_2 *s, long d) {
    s->a = s->a + d;
}
static long classify142_2(int tag, long a, long b) {
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
static long accum142_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard142_2(long x) {
    return x + 3;
}

long f142(long x) {
    long acc = x;
    acc += f082(x + 1);
    acc += f136(x + 2);
    S142_0 s0 = mk142_0(acc);
    bump142_0(&s0, 4);
    acc += probe142_0(&s0);
    acc += read142_0(&s0);
    acc += classify142_0(1, acc, acc);
    acc += accum142_0(6);
    acc += guard142_0(acc);
    S142_1 s1 = mk142_1(acc);
    bump142_1(&s1, 1);
    acc += probe142_1(&s1);
    acc += read142_1(&s1);
    acc += classify142_1(1, acc, acc);
    acc += accum142_1(3);
    acc += guard142_1(acc);
    S142_2 s2 = mk142_2(acc);
    bump142_2(&s2, 2);
    acc += probe142_2(&s2);
    acc += read142_2(&s2);
    acc += classify142_2(1, acc, acc);
    acc += accum142_2(4);
    acc += guard142_2(acc);
    return clampi(acc);
}
