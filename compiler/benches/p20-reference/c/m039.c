/* GENERATED C mirror of reference module m039. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S39_0;

static S39_0 mk39_0(long a) {
    S39_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe39_0(const S39_0 *s) {
    return s->a + s->n0;
}
static long read39_0(const S39_0 *s) {
    return s->a * 4;
}
static void bump39_0(S39_0 *s, long d) {
    s->a = s->a + d;
}
static long classify39_0(int tag, long a, long b) {
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
static long accum39_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard39_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S39_1;

static S39_1 mk39_1(long a) {
    S39_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe39_1(const S39_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read39_1(const S39_1 *s) {
    return s->a * 7;
}
static void bump39_1(S39_1 *s, long d) {
    s->a = s->a + d;
}
static long classify39_1(int tag, long a, long b) {
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
static long accum39_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard39_1(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S39_2;

static S39_2 mk39_2(long a) {
    S39_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe39_2(const S39_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read39_2(const S39_2 *s) {
    return s->a * 4;
}
static void bump39_2(S39_2 *s, long d) {
    s->a = s->a + d;
}
static long classify39_2(int tag, long a, long b) {
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
static long accum39_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard39_2(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S39_3;

static S39_3 mk39_3(long a) {
    S39_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe39_3(const S39_3 *s) {
    return s->a + s->n0;
}
static long read39_3(const S39_3 *s) {
    return s->a * 3;
}
static void bump39_3(S39_3 *s, long d) {
    s->a = s->a + d;
}
static long classify39_3(int tag, long a, long b) {
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
static long accum39_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard39_3(long x) {
    return x + 7;
}

long f039(long x) {
    long acc = x;
    acc += f002(x + 1);
    acc += f004(x + 2);
    acc += f008(x + 3);
    acc += f016(x + 4);
    S39_0 s0 = mk39_0(acc);
    bump39_0(&s0, 3);
    acc += probe39_0(&s0);
    acc += read39_0(&s0);
    acc += classify39_0(1, acc, acc);
    acc += accum39_0(7);
    acc += guard39_0(acc);
    S39_1 s1 = mk39_1(acc);
    bump39_1(&s1, 9);
    acc += probe39_1(&s1);
    acc += read39_1(&s1);
    acc += classify39_1(1, acc, acc);
    acc += accum39_1(6);
    acc += guard39_1(acc);
    S39_2 s2 = mk39_2(acc);
    bump39_2(&s2, 6);
    acc += probe39_2(&s2);
    acc += read39_2(&s2);
    acc += classify39_2(1, acc, acc);
    acc += accum39_2(5);
    acc += guard39_2(acc);
    S39_3 s3 = mk39_3(acc);
    bump39_3(&s3, 1);
    acc += probe39_3(&s3);
    acc += read39_3(&s3);
    acc += classify39_3(1, acc, acc);
    acc += accum39_3(8);
    acc += guard39_3(acc);
    return clampi(acc);
}
