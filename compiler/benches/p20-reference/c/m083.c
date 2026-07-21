/* GENERATED C mirror of reference module m083. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S83_0;

static S83_0 mk83_0(long a) {
    S83_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe83_0(const S83_0 *s) {
    return s->a + s->n0;
}
static long read83_0(const S83_0 *s) {
    return s->a * 2;
}
static void bump83_0(S83_0 *s, long d) {
    s->a = s->a + d;
}
static long classify83_0(int tag, long a, long b) {
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
static long accum83_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard83_0(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S83_1;

static S83_1 mk83_1(long a) {
    S83_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe83_1(const S83_1 *s) {
    return s->a + s->n0;
}
static long read83_1(const S83_1 *s) {
    return s->a * 4;
}
static void bump83_1(S83_1 *s, long d) {
    s->a = s->a + d;
}
static long classify83_1(int tag, long a, long b) {
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
static long accum83_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard83_1(long x) {
    return x + 8;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S83_2;

static S83_2 mk83_2(long a) {
    S83_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe83_2(const S83_2 *s) {
    return s->a + s->n0;
}
static long read83_2(const S83_2 *s) {
    return s->a * 6;
}
static void bump83_2(S83_2 *s, long d) {
    s->a = s->a + d;
}
static long classify83_2(int tag, long a, long b) {
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
static long accum83_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard83_2(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S83_3;

static S83_3 mk83_3(long a) {
    S83_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe83_3(const S83_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read83_3(const S83_3 *s) {
    return s->a * 7;
}
static void bump83_3(S83_3 *s, long d) {
    s->a = s->a + d;
}
static long classify83_3(int tag, long a, long b) {
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
static long accum83_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard83_3(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S83_4;

static S83_4 mk83_4(long a) {
    S83_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe83_4(const S83_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read83_4(const S83_4 *s) {
    return s->a * 5;
}
static void bump83_4(S83_4 *s, long d) {
    s->a = s->a + d;
}
static long classify83_4(int tag, long a, long b) {
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
static long accum83_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard83_4(long x) {
    return x + 3;
}

long f083(long x) {
    long acc = x;
    acc += f029(x + 1);
    S83_0 s0 = mk83_0(acc);
    bump83_0(&s0, 4);
    acc += probe83_0(&s0);
    acc += read83_0(&s0);
    acc += classify83_0(1, acc, acc);
    acc += accum83_0(4);
    acc += guard83_0(acc);
    S83_1 s1 = mk83_1(acc);
    bump83_1(&s1, 1);
    acc += probe83_1(&s1);
    acc += read83_1(&s1);
    acc += classify83_1(1, acc, acc);
    acc += accum83_1(3);
    acc += guard83_1(acc);
    S83_2 s2 = mk83_2(acc);
    bump83_2(&s2, 5);
    acc += probe83_2(&s2);
    acc += read83_2(&s2);
    acc += classify83_2(1, acc, acc);
    acc += accum83_2(7);
    acc += guard83_2(acc);
    S83_3 s3 = mk83_3(acc);
    bump83_3(&s3, 1);
    acc += probe83_3(&s3);
    acc += read83_3(&s3);
    acc += classify83_3(1, acc, acc);
    acc += accum83_3(8);
    acc += guard83_3(acc);
    S83_4 s4 = mk83_4(acc);
    bump83_4(&s4, 5);
    acc += probe83_4(&s4);
    acc += read83_4(&s4);
    acc += classify83_4(1, acc, acc);
    acc += accum83_4(9);
    acc += guard83_4(acc);
    return clampi(acc);
}
