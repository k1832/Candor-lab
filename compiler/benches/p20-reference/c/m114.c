/* GENERATED C mirror of reference module m114. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S114_0;

static S114_0 mk114_0(long a) {
    S114_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe114_0(const S114_0 *s) {
    return s->a + s->n0;
}
static long read114_0(const S114_0 *s) {
    return s->a * 4;
}
static void bump114_0(S114_0 *s, long d) {
    s->a = s->a + d;
}
static long classify114_0(int tag, long a, long b) {
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
static long accum114_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard114_0(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S114_1;

static S114_1 mk114_1(long a) {
    S114_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe114_1(const S114_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read114_1(const S114_1 *s) {
    return s->a * 2;
}
static void bump114_1(S114_1 *s, long d) {
    s->a = s->a + d;
}
static long classify114_1(int tag, long a, long b) {
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
static long accum114_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard114_1(long x) {
    return x + 9;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S114_2;

static S114_2 mk114_2(long a) {
    S114_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe114_2(const S114_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read114_2(const S114_2 *s) {
    return s->a * 2;
}
static void bump114_2(S114_2 *s, long d) {
    s->a = s->a + d;
}
static long classify114_2(int tag, long a, long b) {
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
static long accum114_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard114_2(long x) {
    return x + 9;
}

long f114(long x) {
    long acc = x;
    acc += f008(x + 1);
    acc += f051(x + 2);
    acc += f073(x + 3);
    acc += f098(x + 4);
    S114_0 s0 = mk114_0(acc);
    bump114_0(&s0, 9);
    acc += probe114_0(&s0);
    acc += read114_0(&s0);
    acc += classify114_0(1, acc, acc);
    acc += accum114_0(4);
    acc += guard114_0(acc);
    S114_1 s1 = mk114_1(acc);
    bump114_1(&s1, 9);
    acc += probe114_1(&s1);
    acc += read114_1(&s1);
    acc += classify114_1(1, acc, acc);
    acc += accum114_1(8);
    acc += guard114_1(acc);
    S114_2 s2 = mk114_2(acc);
    bump114_2(&s2, 8);
    acc += probe114_2(&s2);
    acc += read114_2(&s2);
    acc += classify114_2(1, acc, acc);
    acc += accum114_2(6);
    acc += guard114_2(acc);
    return clampi(acc);
}
