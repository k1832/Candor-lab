/* GENERATED C mirror of reference module m113. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S113_0;

static S113_0 mk113_0(long a) {
    S113_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe113_0(const S113_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read113_0(const S113_0 *s) {
    return s->a * 6;
}
static void bump113_0(S113_0 *s, long d) {
    s->a = s->a + d;
}
static long classify113_0(int tag, long a, long b) {
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
static long accum113_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard113_0(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S113_1;

static S113_1 mk113_1(long a) {
    S113_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe113_1(const S113_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read113_1(const S113_1 *s) {
    return s->a * 6;
}
static void bump113_1(S113_1 *s, long d) {
    s->a = s->a + d;
}
static long classify113_1(int tag, long a, long b) {
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
static long accum113_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard113_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S113_2;

static S113_2 mk113_2(long a) {
    S113_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe113_2(const S113_2 *s) {
    return s->a + s->n0;
}
static long read113_2(const S113_2 *s) {
    return s->a * 2;
}
static void bump113_2(S113_2 *s, long d) {
    s->a = s->a + d;
}
static long classify113_2(int tag, long a, long b) {
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
static long accum113_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard113_2(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S113_3;

static S113_3 mk113_3(long a) {
    S113_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe113_3(const S113_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read113_3(const S113_3 *s) {
    return s->a * 7;
}
static void bump113_3(S113_3 *s, long d) {
    s->a = s->a + d;
}
static long classify113_3(int tag, long a, long b) {
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
static long accum113_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard113_3(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S113_4;

static S113_4 mk113_4(long a) {
    S113_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe113_4(const S113_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read113_4(const S113_4 *s) {
    return s->a * 6;
}
static void bump113_4(S113_4 *s, long d) {
    s->a = s->a + d;
}
static long classify113_4(int tag, long a, long b) {
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
static long accum113_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard113_4(long x) {
    return x + 7;
}

long f113(long x) {
    long acc = x;
    acc += f009(x + 1);
    acc += f030(x + 2);
    acc += f031(x + 3);
    acc += f035(x + 4);
    S113_0 s0 = mk113_0(acc);
    bump113_0(&s0, 9);
    acc += probe113_0(&s0);
    acc += read113_0(&s0);
    acc += classify113_0(1, acc, acc);
    acc += accum113_0(7);
    acc += guard113_0(acc);
    S113_1 s1 = mk113_1(acc);
    bump113_1(&s1, 4);
    acc += probe113_1(&s1);
    acc += read113_1(&s1);
    acc += classify113_1(1, acc, acc);
    acc += accum113_1(8);
    acc += guard113_1(acc);
    S113_2 s2 = mk113_2(acc);
    bump113_2(&s2, 6);
    acc += probe113_2(&s2);
    acc += read113_2(&s2);
    acc += classify113_2(1, acc, acc);
    acc += accum113_2(8);
    acc += guard113_2(acc);
    S113_3 s3 = mk113_3(acc);
    bump113_3(&s3, 5);
    acc += probe113_3(&s3);
    acc += read113_3(&s3);
    acc += classify113_3(1, acc, acc);
    acc += accum113_3(3);
    acc += guard113_3(acc);
    S113_4 s4 = mk113_4(acc);
    bump113_4(&s4, 1);
    acc += probe113_4(&s4);
    acc += read113_4(&s4);
    acc += classify113_4(1, acc, acc);
    acc += accum113_4(5);
    acc += guard113_4(acc);
    return clampi(acc);
}
