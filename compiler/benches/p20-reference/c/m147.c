/* GENERATED C mirror of reference module m147. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S147_0;

static S147_0 mk147_0(long a) {
    S147_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe147_0(const S147_0 *s) {
    return s->a + s->n0;
}
static long read147_0(const S147_0 *s) {
    return s->a * 3;
}
static void bump147_0(S147_0 *s, long d) {
    s->a = s->a + d;
}
static long classify147_0(int tag, long a, long b) {
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
static long accum147_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard147_0(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S147_1;

static S147_1 mk147_1(long a) {
    S147_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe147_1(const S147_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read147_1(const S147_1 *s) {
    return s->a * 3;
}
static void bump147_1(S147_1 *s, long d) {
    s->a = s->a + d;
}
static long classify147_1(int tag, long a, long b) {
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
static long accum147_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard147_1(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S147_2;

static S147_2 mk147_2(long a) {
    S147_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe147_2(const S147_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read147_2(const S147_2 *s) {
    return s->a * 5;
}
static void bump147_2(S147_2 *s, long d) {
    s->a = s->a + d;
}
static long classify147_2(int tag, long a, long b) {
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
static long accum147_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard147_2(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S147_3;

static S147_3 mk147_3(long a) {
    S147_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe147_3(const S147_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read147_3(const S147_3 *s) {
    return s->a * 3;
}
static void bump147_3(S147_3 *s, long d) {
    s->a = s->a + d;
}
static long classify147_3(int tag, long a, long b) {
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
static long accum147_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard147_3(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S147_4;

static S147_4 mk147_4(long a) {
    S147_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe147_4(const S147_4 *s) {
    return s->a + s->n0;
}
static long read147_4(const S147_4 *s) {
    return s->a * 4;
}
static void bump147_4(S147_4 *s, long d) {
    s->a = s->a + d;
}
static long classify147_4(int tag, long a, long b) {
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
static long accum147_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard147_4(long x) {
    return x + 1;
}

long f147(long x) {
    long acc = x;
    acc += f052(x + 1);
    acc += f092(x + 2);
    S147_0 s0 = mk147_0(acc);
    bump147_0(&s0, 2);
    acc += probe147_0(&s0);
    acc += read147_0(&s0);
    acc += classify147_0(1, acc, acc);
    acc += accum147_0(8);
    acc += guard147_0(acc);
    S147_1 s1 = mk147_1(acc);
    bump147_1(&s1, 8);
    acc += probe147_1(&s1);
    acc += read147_1(&s1);
    acc += classify147_1(1, acc, acc);
    acc += accum147_1(9);
    acc += guard147_1(acc);
    S147_2 s2 = mk147_2(acc);
    bump147_2(&s2, 9);
    acc += probe147_2(&s2);
    acc += read147_2(&s2);
    acc += classify147_2(1, acc, acc);
    acc += accum147_2(4);
    acc += guard147_2(acc);
    S147_3 s3 = mk147_3(acc);
    bump147_3(&s3, 7);
    acc += probe147_3(&s3);
    acc += read147_3(&s3);
    acc += classify147_3(1, acc, acc);
    acc += accum147_3(5);
    acc += guard147_3(acc);
    S147_4 s4 = mk147_4(acc);
    bump147_4(&s4, 5);
    acc += probe147_4(&s4);
    acc += read147_4(&s4);
    acc += classify147_4(1, acc, acc);
    acc += accum147_4(7);
    acc += guard147_4(acc);
    return clampi(acc);
}
