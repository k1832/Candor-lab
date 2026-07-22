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
    return s->a * 2;
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
        acc += i * 4;
    }
    return acc;
}
static long guard121_0(long x) {
    return x + 9;
}

static long pick121_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S121_1;

static S121_1 mk121_1(long a) {
    S121_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe121_1(const S121_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read121_1(const S121_1 *s) {
    return s->a * 4;
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
    return x + 3;
}

static long pick121_1_0(long a, long b) { return a > b ? a : b; }
static long pick121_1_1(long a, long b) { return a > b ? a : b; }
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
    return s->a * 6;
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
        acc += i * 3;
    }
    return acc;
}
static long guard121_2(long x) {
    return x + 4;
}

static long pick121_2_0(long a, long b) { return a > b ? a : b; }
long f121(long x) {
    long acc = x;
    acc += f022(x + 1);
    acc += f070(x + 2);
    S121_0 s0 = mk121_0(acc);
    bump121_0(&s0, 7);
    acc += probe121_0(&s0);
    acc += read121_0(&s0);
    acc += classify121_0(1, acc, acc);
    acc += accum121_0(5);
    acc += guard121_0(acc);
    acc += pick121_0_0(acc, acc + 9);
    S121_1 s1 = mk121_1(acc);
    bump121_1(&s1, 2);
    acc += probe121_1(&s1);
    acc += read121_1(&s1);
    acc += classify121_1(1, acc, acc);
    acc += accum121_1(7);
    acc += guard121_1(acc);
    acc += pick121_1_0(acc, acc + 7);
    acc += pick121_1_1(acc, acc + 8);
    S121_2 s2 = mk121_2(acc);
    bump121_2(&s2, 7);
    acc += probe121_2(&s2);
    acc += read121_2(&s2);
    acc += classify121_2(1, acc, acc);
    acc += accum121_2(9);
    acc += guard121_2(acc);
    acc += pick121_2_0(acc, acc + 3);
    return clampi(acc);
}
