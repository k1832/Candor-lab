/* GENERATED C mirror of reference module m018. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S18_0;

static S18_0 mk18_0(long a) {
    S18_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe18_0(const S18_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read18_0(const S18_0 *s) {
    return s->a * 3;
}
static void bump18_0(S18_0 *s, long d) {
    s->a = s->a + d;
}
static long classify18_0(int tag, long a, long b) {
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
static long accum18_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard18_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S18_1;

static S18_1 mk18_1(long a) {
    S18_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe18_1(const S18_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read18_1(const S18_1 *s) {
    return s->a * 4;
}
static void bump18_1(S18_1 *s, long d) {
    s->a = s->a + d;
}
static long classify18_1(int tag, long a, long b) {
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
static long accum18_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard18_1(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S18_2;

static S18_2 mk18_2(long a) {
    S18_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe18_2(const S18_2 *s) {
    return s->a + s->n0;
}
static long read18_2(const S18_2 *s) {
    return s->a * 6;
}
static void bump18_2(S18_2 *s, long d) {
    s->a = s->a + d;
}
static long classify18_2(int tag, long a, long b) {
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
static long accum18_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard18_2(long x) {
    return x + 1;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S18_3;

static S18_3 mk18_3(long a) {
    S18_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe18_3(const S18_3 *s) {
    return s->a + s->n0 + s->n1;
}
static long read18_3(const S18_3 *s) {
    return s->a * 2;
}
static void bump18_3(S18_3 *s, long d) {
    s->a = s->a + d;
}
static long classify18_3(int tag, long a, long b) {
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
static long accum18_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard18_3(long x) {
    return x + 6;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S18_4;

static S18_4 mk18_4(long a) {
    S18_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe18_4(const S18_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read18_4(const S18_4 *s) {
    return s->a * 6;
}
static void bump18_4(S18_4 *s, long d) {
    s->a = s->a + d;
}
static long classify18_4(int tag, long a, long b) {
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
static long accum18_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard18_4(long x) {
    return x + 6;
}

long f018(long x) {
    long acc = x;
    acc += f005(x + 1);
    S18_0 s0 = mk18_0(acc);
    bump18_0(&s0, 9);
    acc += probe18_0(&s0);
    acc += read18_0(&s0);
    acc += classify18_0(1, acc, acc);
    acc += accum18_0(6);
    acc += guard18_0(acc);
    S18_1 s1 = mk18_1(acc);
    bump18_1(&s1, 1);
    acc += probe18_1(&s1);
    acc += read18_1(&s1);
    acc += classify18_1(1, acc, acc);
    acc += accum18_1(7);
    acc += guard18_1(acc);
    S18_2 s2 = mk18_2(acc);
    bump18_2(&s2, 2);
    acc += probe18_2(&s2);
    acc += read18_2(&s2);
    acc += classify18_2(1, acc, acc);
    acc += accum18_2(3);
    acc += guard18_2(acc);
    S18_3 s3 = mk18_3(acc);
    bump18_3(&s3, 3);
    acc += probe18_3(&s3);
    acc += read18_3(&s3);
    acc += classify18_3(1, acc, acc);
    acc += accum18_3(9);
    acc += guard18_3(acc);
    S18_4 s4 = mk18_4(acc);
    bump18_4(&s4, 1);
    acc += probe18_4(&s4);
    acc += read18_4(&s4);
    acc += classify18_4(1, acc, acc);
    acc += accum18_4(8);
    acc += guard18_4(acc);
    return clampi(acc);
}
