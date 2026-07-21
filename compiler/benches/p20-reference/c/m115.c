/* GENERATED C mirror of reference module m115. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S115_0;

static S115_0 mk115_0(long a) {
    S115_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe115_0(const S115_0 *s) {
    return s->a + s->n0;
}
static long read115_0(const S115_0 *s) {
    return s->a * 6;
}
static void bump115_0(S115_0 *s, long d) {
    s->a = s->a + d;
}
static long classify115_0(int tag, long a, long b) {
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
static long accum115_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard115_0(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S115_1;

static S115_1 mk115_1(long a) {
    S115_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe115_1(const S115_1 *s) {
    return s->a + s->n0;
}
static long read115_1(const S115_1 *s) {
    return s->a * 4;
}
static void bump115_1(S115_1 *s, long d) {
    s->a = s->a + d;
}
static long classify115_1(int tag, long a, long b) {
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
static long accum115_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard115_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S115_2;

static S115_2 mk115_2(long a) {
    S115_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe115_2(const S115_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read115_2(const S115_2 *s) {
    return s->a * 7;
}
static void bump115_2(S115_2 *s, long d) {
    s->a = s->a + d;
}
static long classify115_2(int tag, long a, long b) {
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
static long accum115_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard115_2(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S115_3;

static S115_3 mk115_3(long a) {
    S115_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe115_3(const S115_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read115_3(const S115_3 *s) {
    return s->a * 4;
}
static void bump115_3(S115_3 *s, long d) {
    s->a = s->a + d;
}
static long classify115_3(int tag, long a, long b) {
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
static long accum115_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard115_3(long x) {
    return x + 7;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S115_4;

static S115_4 mk115_4(long a) {
    S115_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe115_4(const S115_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read115_4(const S115_4 *s) {
    return s->a * 6;
}
static void bump115_4(S115_4 *s, long d) {
    s->a = s->a + d;
}
static long classify115_4(int tag, long a, long b) {
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
static long accum115_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard115_4(long x) {
    return x + 7;
}

long f115(long x) {
    long acc = x;
    acc += f044(x + 1);
    S115_0 s0 = mk115_0(acc);
    bump115_0(&s0, 7);
    acc += probe115_0(&s0);
    acc += read115_0(&s0);
    acc += classify115_0(1, acc, acc);
    acc += accum115_0(5);
    acc += guard115_0(acc);
    S115_1 s1 = mk115_1(acc);
    bump115_1(&s1, 4);
    acc += probe115_1(&s1);
    acc += read115_1(&s1);
    acc += classify115_1(1, acc, acc);
    acc += accum115_1(3);
    acc += guard115_1(acc);
    S115_2 s2 = mk115_2(acc);
    bump115_2(&s2, 4);
    acc += probe115_2(&s2);
    acc += read115_2(&s2);
    acc += classify115_2(1, acc, acc);
    acc += accum115_2(3);
    acc += guard115_2(acc);
    S115_3 s3 = mk115_3(acc);
    bump115_3(&s3, 8);
    acc += probe115_3(&s3);
    acc += read115_3(&s3);
    acc += classify115_3(1, acc, acc);
    acc += accum115_3(8);
    acc += guard115_3(acc);
    S115_4 s4 = mk115_4(acc);
    bump115_4(&s4, 5);
    acc += probe115_4(&s4);
    acc += read115_4(&s4);
    acc += classify115_4(1, acc, acc);
    acc += accum115_4(5);
    acc += guard115_4(acc);
    return clampi(acc);
}
