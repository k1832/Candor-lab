/* GENERATED C mirror of reference module m153. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S153_0;

static S153_0 mk153_0(long a) {
    S153_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe153_0(const S153_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read153_0(const S153_0 *s) {
    return s->a * 6;
}
static void bump153_0(S153_0 *s, long d) {
    s->a = s->a + d;
}
static long classify153_0(int tag, long a, long b) {
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
static long accum153_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard153_0(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S153_1;

static S153_1 mk153_1(long a) {
    S153_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe153_1(const S153_1 *s) {
    return s->a + s->n0;
}
static long read153_1(const S153_1 *s) {
    return s->a * 5;
}
static void bump153_1(S153_1 *s, long d) {
    s->a = s->a + d;
}
static long classify153_1(int tag, long a, long b) {
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
static long accum153_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard153_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S153_2;

static S153_2 mk153_2(long a) {
    S153_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe153_2(const S153_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read153_2(const S153_2 *s) {
    return s->a * 5;
}
static void bump153_2(S153_2 *s, long d) {
    s->a = s->a + d;
}
static long classify153_2(int tag, long a, long b) {
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
static long accum153_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard153_2(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S153_3;

static S153_3 mk153_3(long a) {
    S153_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe153_3(const S153_3 *s) {
    return s->a + s->n0;
}
static long read153_3(const S153_3 *s) {
    return s->a * 5;
}
static void bump153_3(S153_3 *s, long d) {
    s->a = s->a + d;
}
static long classify153_3(int tag, long a, long b) {
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
static long accum153_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard153_3(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S153_4;

static S153_4 mk153_4(long a) {
    S153_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe153_4(const S153_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read153_4(const S153_4 *s) {
    return s->a * 5;
}
static void bump153_4(S153_4 *s, long d) {
    s->a = s->a + d;
}
static long classify153_4(int tag, long a, long b) {
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
static long accum153_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard153_4(long x) {
    return x + 6;
}

long f153(long x) {
    long acc = x;
    acc += f003(x + 1);
    S153_0 s0 = mk153_0(acc);
    bump153_0(&s0, 9);
    acc += probe153_0(&s0);
    acc += read153_0(&s0);
    acc += classify153_0(1, acc, acc);
    acc += accum153_0(7);
    acc += guard153_0(acc);
    S153_1 s1 = mk153_1(acc);
    bump153_1(&s1, 9);
    acc += probe153_1(&s1);
    acc += read153_1(&s1);
    acc += classify153_1(1, acc, acc);
    acc += accum153_1(5);
    acc += guard153_1(acc);
    S153_2 s2 = mk153_2(acc);
    bump153_2(&s2, 6);
    acc += probe153_2(&s2);
    acc += read153_2(&s2);
    acc += classify153_2(1, acc, acc);
    acc += accum153_2(4);
    acc += guard153_2(acc);
    S153_3 s3 = mk153_3(acc);
    bump153_3(&s3, 1);
    acc += probe153_3(&s3);
    acc += read153_3(&s3);
    acc += classify153_3(1, acc, acc);
    acc += accum153_3(9);
    acc += guard153_3(acc);
    S153_4 s4 = mk153_4(acc);
    bump153_4(&s4, 2);
    acc += probe153_4(&s4);
    acc += read153_4(&s4);
    acc += classify153_4(1, acc, acc);
    acc += accum153_4(5);
    acc += guard153_4(acc);
    return clampi(acc);
}
