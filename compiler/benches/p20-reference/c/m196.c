/* GENERATED C mirror of reference module m196. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S196_0;

static S196_0 mk196_0(long a) {
    S196_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe196_0(const S196_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read196_0(const S196_0 *s) {
    return s->a * 6;
}
static void bump196_0(S196_0 *s, long d) {
    s->a = s->a + d;
}
static long classify196_0(int tag, long a, long b) {
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
static long accum196_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard196_0(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S196_1;

static S196_1 mk196_1(long a) {
    S196_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe196_1(const S196_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read196_1(const S196_1 *s) {
    return s->a * 7;
}
static void bump196_1(S196_1 *s, long d) {
    s->a = s->a + d;
}
static long classify196_1(int tag, long a, long b) {
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
static long accum196_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard196_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S196_2;

static S196_2 mk196_2(long a) {
    S196_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe196_2(const S196_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read196_2(const S196_2 *s) {
    return s->a * 7;
}
static void bump196_2(S196_2 *s, long d) {
    s->a = s->a + d;
}
static long classify196_2(int tag, long a, long b) {
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
static long accum196_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard196_2(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S196_3;

static S196_3 mk196_3(long a) {
    S196_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe196_3(const S196_3 *s) {
    return s->a + s->n0;
}
static long read196_3(const S196_3 *s) {
    return s->a * 5;
}
static void bump196_3(S196_3 *s, long d) {
    s->a = s->a + d;
}
static long classify196_3(int tag, long a, long b) {
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
static long accum196_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard196_3(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S196_4;

static S196_4 mk196_4(long a) {
    S196_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe196_4(const S196_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read196_4(const S196_4 *s) {
    return s->a * 7;
}
static void bump196_4(S196_4 *s, long d) {
    s->a = s->a + d;
}
static long classify196_4(int tag, long a, long b) {
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
static long accum196_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard196_4(long x) {
    return x + 8;
}

long f196(long x) {
    long acc = x;
    acc += f122(x + 1);
    acc += f163(x + 2);
    acc += f187(x + 3);
    S196_0 s0 = mk196_0(acc);
    bump196_0(&s0, 7);
    acc += probe196_0(&s0);
    acc += read196_0(&s0);
    acc += classify196_0(1, acc, acc);
    acc += accum196_0(8);
    acc += guard196_0(acc);
    S196_1 s1 = mk196_1(acc);
    bump196_1(&s1, 8);
    acc += probe196_1(&s1);
    acc += read196_1(&s1);
    acc += classify196_1(1, acc, acc);
    acc += accum196_1(5);
    acc += guard196_1(acc);
    S196_2 s2 = mk196_2(acc);
    bump196_2(&s2, 4);
    acc += probe196_2(&s2);
    acc += read196_2(&s2);
    acc += classify196_2(1, acc, acc);
    acc += accum196_2(5);
    acc += guard196_2(acc);
    S196_3 s3 = mk196_3(acc);
    bump196_3(&s3, 9);
    acc += probe196_3(&s3);
    acc += read196_3(&s3);
    acc += classify196_3(1, acc, acc);
    acc += accum196_3(9);
    acc += guard196_3(acc);
    S196_4 s4 = mk196_4(acc);
    bump196_4(&s4, 1);
    acc += probe196_4(&s4);
    acc += read196_4(&s4);
    acc += classify196_4(1, acc, acc);
    acc += accum196_4(4);
    acc += guard196_4(acc);
    return clampi(acc);
}
