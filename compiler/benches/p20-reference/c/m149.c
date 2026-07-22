/* GENERATED C mirror of reference module m149. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S149_0;

static S149_0 mk149_0(long a) {
    S149_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe149_0(const S149_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read149_0(const S149_0 *s) {
    return s->a * 7;
}
static void bump149_0(S149_0 *s, long d) {
    s->a = s->a + d;
}
static long classify149_0(int tag, long a, long b) {
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
static long accum149_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard149_0(long x) {
    return x + 7;
}

static long pick149_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S149_1;

static S149_1 mk149_1(long a) {
    S149_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe149_1(const S149_1 *s) {
    return s->a + s->n0;
}
static long read149_1(const S149_1 *s) {
    return s->a * 4;
}
static void bump149_1(S149_1 *s, long d) {
    s->a = s->a + d;
}
static long classify149_1(int tag, long a, long b) {
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
static long accum149_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard149_1(long x) {
    return x + 4;
}

static long pick149_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S149_2;

static S149_2 mk149_2(long a) {
    S149_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe149_2(const S149_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read149_2(const S149_2 *s) {
    return s->a * 3;
}
static void bump149_2(S149_2 *s, long d) {
    s->a = s->a + d;
}
static long classify149_2(int tag, long a, long b) {
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
static long accum149_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard149_2(long x) {
    return x + 1;
}

static long pick149_2_0(long a, long b) { return a > b ? a : b; }
static long pick149_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S149_3;

static S149_3 mk149_3(long a) {
    S149_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe149_3(const S149_3 *s) {
    return s->a + s->n0;
}
static long read149_3(const S149_3 *s) {
    return s->a * 4;
}
static void bump149_3(S149_3 *s, long d) {
    s->a = s->a + d;
}
static long classify149_3(int tag, long a, long b) {
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
static long accum149_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard149_3(long x) {
    return x + 1;
}

static long pick149_3_0(long a, long b) { return a > b ? a : b; }
static long pick149_3_1(long a, long b) { return a > b ? a : b; }
static long pick149_3_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S149_4;

static S149_4 mk149_4(long a) {
    S149_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe149_4(const S149_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read149_4(const S149_4 *s) {
    return s->a * 4;
}
static void bump149_4(S149_4 *s, long d) {
    s->a = s->a + d;
}
static long classify149_4(int tag, long a, long b) {
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
static long accum149_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard149_4(long x) {
    return x + 4;
}

static long pick149_4_0(long a, long b) { return a > b ? a : b; }
static long pick149_4_1(long a, long b) { return a > b ? a : b; }
static long pick149_4_2(long a, long b) { return a > b ? a : b; }
long f149(long x) {
    long acc = x;
    acc += f070(x + 1);
    S149_0 s0 = mk149_0(acc);
    bump149_0(&s0, 7);
    acc += probe149_0(&s0);
    acc += read149_0(&s0);
    acc += classify149_0(1, acc, acc);
    acc += accum149_0(4);
    acc += guard149_0(acc);
    acc += pick149_0_0(acc, acc + 3);
    S149_1 s1 = mk149_1(acc);
    bump149_1(&s1, 5);
    acc += probe149_1(&s1);
    acc += read149_1(&s1);
    acc += classify149_1(1, acc, acc);
    acc += accum149_1(7);
    acc += guard149_1(acc);
    acc += pick149_1_0(acc, acc + 6);
    S149_2 s2 = mk149_2(acc);
    bump149_2(&s2, 6);
    acc += probe149_2(&s2);
    acc += read149_2(&s2);
    acc += classify149_2(1, acc, acc);
    acc += accum149_2(9);
    acc += guard149_2(acc);
    acc += pick149_2_0(acc, acc + 5);
    acc += pick149_2_1(acc, acc + 1);
    S149_3 s3 = mk149_3(acc);
    bump149_3(&s3, 9);
    acc += probe149_3(&s3);
    acc += read149_3(&s3);
    acc += classify149_3(1, acc, acc);
    acc += accum149_3(3);
    acc += guard149_3(acc);
    acc += pick149_3_0(acc, acc + 9);
    acc += pick149_3_1(acc, acc + 7);
    acc += pick149_3_2(acc, acc + 6);
    S149_4 s4 = mk149_4(acc);
    bump149_4(&s4, 4);
    acc += probe149_4(&s4);
    acc += read149_4(&s4);
    acc += classify149_4(1, acc, acc);
    acc += accum149_4(3);
    acc += guard149_4(acc);
    acc += pick149_4_0(acc, acc + 2);
    acc += pick149_4_1(acc, acc + 1);
    acc += pick149_4_2(acc, acc + 4);
    return clampi(acc);
}
