/* GENERATED C mirror of reference module m170. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S170_0;

static S170_0 mk170_0(long a) {
    S170_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe170_0(const S170_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read170_0(const S170_0 *s) {
    return s->a * 6;
}
static void bump170_0(S170_0 *s, long d) {
    s->a = s->a + d;
}
static long classify170_0(int tag, long a, long b) {
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
static long accum170_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard170_0(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S170_1;

static S170_1 mk170_1(long a) {
    S170_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe170_1(const S170_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read170_1(const S170_1 *s) {
    return s->a * 4;
}
static void bump170_1(S170_1 *s, long d) {
    s->a = s->a + d;
}
static long classify170_1(int tag, long a, long b) {
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
static long accum170_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard170_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S170_2;

static S170_2 mk170_2(long a) {
    S170_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe170_2(const S170_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read170_2(const S170_2 *s) {
    return s->a * 7;
}
static void bump170_2(S170_2 *s, long d) {
    s->a = s->a + d;
}
static long classify170_2(int tag, long a, long b) {
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
static long accum170_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard170_2(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S170_3;

static S170_3 mk170_3(long a) {
    S170_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe170_3(const S170_3 *s) {
    return s->a + s->n0;
}
static long read170_3(const S170_3 *s) {
    return s->a * 4;
}
static void bump170_3(S170_3 *s, long d) {
    s->a = s->a + d;
}
static long classify170_3(int tag, long a, long b) {
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
static long accum170_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard170_3(long x) {
    return x + 2;
}

long f170(long x) {
    long acc = x;
    acc += f005(x + 1);
    S170_0 s0 = mk170_0(acc);
    bump170_0(&s0, 4);
    acc += probe170_0(&s0);
    acc += read170_0(&s0);
    acc += classify170_0(1, acc, acc);
    acc += accum170_0(7);
    acc += guard170_0(acc);
    S170_1 s1 = mk170_1(acc);
    bump170_1(&s1, 5);
    acc += probe170_1(&s1);
    acc += read170_1(&s1);
    acc += classify170_1(1, acc, acc);
    acc += accum170_1(5);
    acc += guard170_1(acc);
    S170_2 s2 = mk170_2(acc);
    bump170_2(&s2, 2);
    acc += probe170_2(&s2);
    acc += read170_2(&s2);
    acc += classify170_2(1, acc, acc);
    acc += accum170_2(4);
    acc += guard170_2(acc);
    S170_3 s3 = mk170_3(acc);
    bump170_3(&s3, 2);
    acc += probe170_3(&s3);
    acc += read170_3(&s3);
    acc += classify170_3(1, acc, acc);
    acc += accum170_3(9);
    acc += guard170_3(acc);
    return clampi(acc);
}
