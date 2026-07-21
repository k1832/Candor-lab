/* GENERATED C mirror of reference module m160. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S160_0;

static S160_0 mk160_0(long a) {
    S160_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe160_0(const S160_0 *s) {
    return s->a + s->n0;
}
static long read160_0(const S160_0 *s) {
    return s->a * 4;
}
static void bump160_0(S160_0 *s, long d) {
    s->a = s->a + d;
}
static long classify160_0(int tag, long a, long b) {
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
static long accum160_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard160_0(long x) {
    return x + 2;
}

typedef struct {
    long a;
    long n0;
    int flag;
} S160_1;

static S160_1 mk160_1(long a) {
    S160_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe160_1(const S160_1 *s) {
    return s->a + s->n0;
}
static long read160_1(const S160_1 *s) {
    return s->a * 2;
}
static void bump160_1(S160_1 *s, long d) {
    s->a = s->a + d;
}
static long classify160_1(int tag, long a, long b) {
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
static long accum160_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard160_1(long x) {
    return x + 4;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S160_2;

static S160_2 mk160_2(long a) {
    S160_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe160_2(const S160_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read160_2(const S160_2 *s) {
    return s->a * 4;
}
static void bump160_2(S160_2 *s, long d) {
    s->a = s->a + d;
}
static long classify160_2(int tag, long a, long b) {
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
static long accum160_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard160_2(long x) {
    return x + 6;
}

long f160(long x) {
    long acc = x;
    acc += f091(x + 1);
    acc += f130(x + 2);
    S160_0 s0 = mk160_0(acc);
    bump160_0(&s0, 9);
    acc += probe160_0(&s0);
    acc += read160_0(&s0);
    acc += classify160_0(1, acc, acc);
    acc += accum160_0(8);
    acc += guard160_0(acc);
    S160_1 s1 = mk160_1(acc);
    bump160_1(&s1, 6);
    acc += probe160_1(&s1);
    acc += read160_1(&s1);
    acc += classify160_1(1, acc, acc);
    acc += accum160_1(6);
    acc += guard160_1(acc);
    S160_2 s2 = mk160_2(acc);
    bump160_2(&s2, 3);
    acc += probe160_2(&s2);
    acc += read160_2(&s2);
    acc += classify160_2(1, acc, acc);
    acc += accum160_2(4);
    acc += guard160_2(acc);
    return clampi(acc);
}
