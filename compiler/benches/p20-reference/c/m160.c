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
    return s->a * 7;
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
    return x + 8;
}

static long pick160_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S160_1;

static S160_1 mk160_1(long a) {
    S160_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe160_1(const S160_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read160_1(const S160_1 *s) {
    return s->a * 7;
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
    return x + 7;
}

static long pick160_1_0(long a, long b) { return a > b ? a : b; }
static long pick160_1_1(long a, long b) { return a > b ? a : b; }
static long pick160_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S160_2;

static S160_2 mk160_2(long a) {
    S160_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe160_2(const S160_2 *s) {
    return s->a + s->n0;
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
        acc += i * 3;
    }
    return acc;
}
static long guard160_2(long x) {
    return x + 1;
}

static long pick160_2_0(long a, long b) { return a > b ? a : b; }
long f160(long x) {
    long acc = x;
    acc += f038(x + 1);
    acc += f049(x + 2);
    acc += f114(x + 3);
    acc += f117(x + 4);
    S160_0 s0 = mk160_0(acc);
    bump160_0(&s0, 9);
    acc += probe160_0(&s0);
    acc += read160_0(&s0);
    acc += classify160_0(1, acc, acc);
    acc += accum160_0(6);
    acc += guard160_0(acc);
    acc += pick160_0_0(acc, acc + 7);
    S160_1 s1 = mk160_1(acc);
    bump160_1(&s1, 3);
    acc += probe160_1(&s1);
    acc += read160_1(&s1);
    acc += classify160_1(1, acc, acc);
    acc += accum160_1(6);
    acc += guard160_1(acc);
    acc += pick160_1_0(acc, acc + 7);
    acc += pick160_1_1(acc, acc + 1);
    acc += pick160_1_2(acc, acc + 6);
    S160_2 s2 = mk160_2(acc);
    bump160_2(&s2, 2);
    acc += probe160_2(&s2);
    acc += read160_2(&s2);
    acc += classify160_2(1, acc, acc);
    acc += accum160_2(9);
    acc += guard160_2(acc);
    acc += pick160_2_0(acc, acc + 8);
    return clampi(acc);
}
