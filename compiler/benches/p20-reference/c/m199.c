/* GENERATED C mirror of reference module m199. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S199_0;

static S199_0 mk199_0(long a) {
    S199_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe199_0(const S199_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read199_0(const S199_0 *s) {
    return s->a * 7;
}
static void bump199_0(S199_0 *s, long d) {
    s->a = s->a + d;
}
static long classify199_0(int tag, long a, long b) {
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
static long accum199_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard199_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S199_1;

static S199_1 mk199_1(long a) {
    S199_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe199_1(const S199_1 *s) {
    return s->a + s->n0;
}
static long read199_1(const S199_1 *s) {
    return s->a * 7;
}
static void bump199_1(S199_1 *s, long d) {
    s->a = s->a + d;
}
static long classify199_1(int tag, long a, long b) {
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
static long accum199_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard199_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S199_2;

static S199_2 mk199_2(long a) {
    S199_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe199_2(const S199_2 *s) {
    return s->a + s->n0;
}
static long read199_2(const S199_2 *s) {
    return s->a * 2;
}
static void bump199_2(S199_2 *s, long d) {
    s->a = s->a + d;
}
static long classify199_2(int tag, long a, long b) {
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
static long accum199_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard199_2(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S199_3;

static S199_3 mk199_3(long a) {
    S199_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe199_3(const S199_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read199_3(const S199_3 *s) {
    return s->a * 2;
}
static void bump199_3(S199_3 *s, long d) {
    s->a = s->a + d;
}
static long classify199_3(int tag, long a, long b) {
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
static long accum199_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard199_3(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S199_4;

static S199_4 mk199_4(long a) {
    S199_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe199_4(const S199_4 *s) {
    return s->a + s->n0;
}
static long read199_4(const S199_4 *s) {
    return s->a * 4;
}
static void bump199_4(S199_4 *s, long d) {
    s->a = s->a + d;
}
static long classify199_4(int tag, long a, long b) {
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
static long accum199_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard199_4(long x) {
    return x + 7;
}

long f199(long x) {
    long acc = x;
    acc += f009(x + 1);
    acc += f176(x + 2);
    S199_0 s0 = mk199_0(acc);
    bump199_0(&s0, 4);
    acc += probe199_0(&s0);
    acc += read199_0(&s0);
    acc += classify199_0(1, acc, acc);
    acc += accum199_0(9);
    acc += guard199_0(acc);
    S199_1 s1 = mk199_1(acc);
    bump199_1(&s1, 4);
    acc += probe199_1(&s1);
    acc += read199_1(&s1);
    acc += classify199_1(1, acc, acc);
    acc += accum199_1(6);
    acc += guard199_1(acc);
    S199_2 s2 = mk199_2(acc);
    bump199_2(&s2, 9);
    acc += probe199_2(&s2);
    acc += read199_2(&s2);
    acc += classify199_2(1, acc, acc);
    acc += accum199_2(6);
    acc += guard199_2(acc);
    S199_3 s3 = mk199_3(acc);
    bump199_3(&s3, 6);
    acc += probe199_3(&s3);
    acc += read199_3(&s3);
    acc += classify199_3(1, acc, acc);
    acc += accum199_3(4);
    acc += guard199_3(acc);
    S199_4 s4 = mk199_4(acc);
    bump199_4(&s4, 5);
    acc += probe199_4(&s4);
    acc += read199_4(&s4);
    acc += classify199_4(1, acc, acc);
    acc += accum199_4(8);
    acc += guard199_4(acc);
    return clampi(acc);
}
