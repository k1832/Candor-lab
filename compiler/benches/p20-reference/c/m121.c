/* GENERATED C mirror of reference module m121. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S121_0;

static S121_0 mk121_0(long a) {
    S121_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe121_0(const S121_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read121_0(const S121_0 *s) {
    return s->a * 6;
}
static void bump121_0(S121_0 *s, long d) {
    s->a = s->a + d;
}
static long classify121_0(int tag, long a, long b) {
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
static long accum121_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard121_0(long x) {
    return x + 3;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S121_1;

static S121_1 mk121_1(long a) {
    S121_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe121_1(const S121_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read121_1(const S121_1 *s) {
    return s->a * 7;
}
static void bump121_1(S121_1 *s, long d) {
    s->a = s->a + d;
}
static long classify121_1(int tag, long a, long b) {
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
static long accum121_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard121_1(long x) {
    return x + 5;
}

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S121_2;

static S121_2 mk121_2(long a) {
    S121_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe121_2(const S121_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read121_2(const S121_2 *s) {
    return s->a * 5;
}
static void bump121_2(S121_2 *s, long d) {
    s->a = s->a + d;
}
static long classify121_2(int tag, long a, long b) {
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
static long accum121_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard121_2(long x) {
    return x + 3;
}

long f121(long x) {
    long acc = x;
    acc += f005(x + 1);
    acc += f065(x + 2);
    acc += f105(x + 3);
    S121_0 s0 = mk121_0(acc);
    bump121_0(&s0, 9);
    acc += probe121_0(&s0);
    acc += read121_0(&s0);
    acc += classify121_0(1, acc, acc);
    acc += accum121_0(7);
    acc += guard121_0(acc);
    S121_1 s1 = mk121_1(acc);
    bump121_1(&s1, 5);
    acc += probe121_1(&s1);
    acc += read121_1(&s1);
    acc += classify121_1(1, acc, acc);
    acc += accum121_1(3);
    acc += guard121_1(acc);
    S121_2 s2 = mk121_2(acc);
    bump121_2(&s2, 8);
    acc += probe121_2(&s2);
    acc += read121_2(&s2);
    acc += classify121_2(1, acc, acc);
    acc += accum121_2(5);
    acc += guard121_2(acc);
    return clampi(acc);
}
