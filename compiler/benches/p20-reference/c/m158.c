/* GENERATED C mirror of reference module m158. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S158_0;

static S158_0 mk158_0(long a) {
    S158_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe158_0(const S158_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read158_0(const S158_0 *s) {
    return s->a * 3;
}
static void bump158_0(S158_0 *s, long d) {
    s->a = s->a + d;
}
static long classify158_0(int tag, long a, long b) {
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
static long accum158_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard158_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S158_1;

static S158_1 mk158_1(long a) {
    S158_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe158_1(const S158_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read158_1(const S158_1 *s) {
    return s->a * 4;
}
static void bump158_1(S158_1 *s, long d) {
    s->a = s->a + d;
}
static long classify158_1(int tag, long a, long b) {
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
static long accum158_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard158_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S158_2;

static S158_2 mk158_2(long a) {
    S158_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe158_2(const S158_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read158_2(const S158_2 *s) {
    return s->a * 3;
}
static void bump158_2(S158_2 *s, long d) {
    s->a = s->a + d;
}
static long classify158_2(int tag, long a, long b) {
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
static long accum158_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard158_2(long x) {
    return x + 8;
}

long f158(long x) {
    long acc = x;
    acc += f006(x + 1);
    acc += f050(x + 2);
    acc += f076(x + 3);
    acc += f123(x + 4);
    S158_0 s0 = mk158_0(acc);
    bump158_0(&s0, 9);
    acc += probe158_0(&s0);
    acc += read158_0(&s0);
    acc += classify158_0(1, acc, acc);
    acc += accum158_0(7);
    acc += guard158_0(acc);
    S158_1 s1 = mk158_1(acc);
    bump158_1(&s1, 8);
    acc += probe158_1(&s1);
    acc += read158_1(&s1);
    acc += classify158_1(1, acc, acc);
    acc += accum158_1(3);
    acc += guard158_1(acc);
    S158_2 s2 = mk158_2(acc);
    bump158_2(&s2, 8);
    acc += probe158_2(&s2);
    acc += read158_2(&s2);
    acc += classify158_2(1, acc, acc);
    acc += accum158_2(9);
    acc += guard158_2(acc);
    return clampi(acc);
}
