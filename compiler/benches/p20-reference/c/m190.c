/* GENERATED C mirror of reference module m190. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S190_0;

static S190_0 mk190_0(long a) {
    S190_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe190_0(const S190_0 *s) {
    return s->a + s->n0;
}
static long read190_0(const S190_0 *s) {
    return s->a * 4;
}
static void bump190_0(S190_0 *s, long d) {
    s->a = s->a + d;
}
static long classify190_0(int tag, long a, long b) {
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
static long accum190_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard190_0(long x) {
    return x + 6;
}

static long pick190_0_0(long a, long b) { return a > b ? a : b; }
static long pick190_0_1(long a, long b) { return a > b ? a : b; }
static long pick190_0_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S190_1;

static S190_1 mk190_1(long a) {
    S190_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe190_1(const S190_1 *s) {
    return s->a + s->n0;
}
static long read190_1(const S190_1 *s) {
    return s->a * 7;
}
static void bump190_1(S190_1 *s, long d) {
    s->a = s->a + d;
}
static long classify190_1(int tag, long a, long b) {
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
static long accum190_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard190_1(long x) {
    return x + 2;
}

static long pick190_1_0(long a, long b) { return a > b ? a : b; }
static long pick190_1_1(long a, long b) { return a > b ? a : b; }
static long pick190_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S190_2;

static S190_2 mk190_2(long a) {
    S190_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe190_2(const S190_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read190_2(const S190_2 *s) {
    return s->a * 4;
}
static void bump190_2(S190_2 *s, long d) {
    s->a = s->a + d;
}
static long classify190_2(int tag, long a, long b) {
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
static long accum190_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard190_2(long x) {
    return x + 5;
}

static long pick190_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S190_3;

static S190_3 mk190_3(long a) {
    S190_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe190_3(const S190_3 *s) {
    return s->a + s->n0;
}
static long read190_3(const S190_3 *s) {
    return s->a * 6;
}
static void bump190_3(S190_3 *s, long d) {
    s->a = s->a + d;
}
static long classify190_3(int tag, long a, long b) {
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
static long accum190_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard190_3(long x) {
    return x + 4;
}

static long pick190_3_0(long a, long b) { return a > b ? a : b; }
static long pick190_3_1(long a, long b) { return a > b ? a : b; }
long f190(long x) {
    long acc = x;
    acc += f055(x + 1);
    S190_0 s0 = mk190_0(acc);
    bump190_0(&s0, 4);
    acc += probe190_0(&s0);
    acc += read190_0(&s0);
    acc += classify190_0(1, acc, acc);
    acc += accum190_0(3);
    acc += guard190_0(acc);
    acc += pick190_0_0(acc, acc + 3);
    acc += pick190_0_1(acc, acc + 2);
    acc += pick190_0_2(acc, acc + 2);
    S190_1 s1 = mk190_1(acc);
    bump190_1(&s1, 5);
    acc += probe190_1(&s1);
    acc += read190_1(&s1);
    acc += classify190_1(1, acc, acc);
    acc += accum190_1(9);
    acc += guard190_1(acc);
    acc += pick190_1_0(acc, acc + 1);
    acc += pick190_1_1(acc, acc + 2);
    acc += pick190_1_2(acc, acc + 5);
    S190_2 s2 = mk190_2(acc);
    bump190_2(&s2, 3);
    acc += probe190_2(&s2);
    acc += read190_2(&s2);
    acc += classify190_2(1, acc, acc);
    acc += accum190_2(9);
    acc += guard190_2(acc);
    acc += pick190_2_0(acc, acc + 9);
    S190_3 s3 = mk190_3(acc);
    bump190_3(&s3, 2);
    acc += probe190_3(&s3);
    acc += read190_3(&s3);
    acc += classify190_3(1, acc, acc);
    acc += accum190_3(8);
    acc += guard190_3(acc);
    acc += pick190_3_0(acc, acc + 5);
    acc += pick190_3_1(acc, acc + 6);
    return clampi(acc);
}
