/* GENERATED C mirror of reference module m116. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S116_0;

static S116_0 mk116_0(long a) {
    S116_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe116_0(const S116_0 *s) {
    return s->a + s->n0 + s->n1;
}
static long read116_0(const S116_0 *s) {
    return s->a * 2;
}
static void bump116_0(S116_0 *s, long d) {
    s->a = s->a + d;
}
static long classify116_0(int tag, long a, long b) {
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
static long accum116_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard116_0(long x) {
    return x + 7;
}

static long pick116_0_0(long a, long b) { return a > b ? a : b; }
static long pick116_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S116_1;

static S116_1 mk116_1(long a) {
    S116_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe116_1(const S116_1 *s) {
    return s->a + s->n0;
}
static long read116_1(const S116_1 *s) {
    return s->a * 4;
}
static void bump116_1(S116_1 *s, long d) {
    s->a = s->a + d;
}
static long classify116_1(int tag, long a, long b) {
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
static long accum116_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard116_1(long x) {
    return x + 6;
}

static long pick116_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S116_2;

static S116_2 mk116_2(long a) {
    S116_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe116_2(const S116_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read116_2(const S116_2 *s) {
    return s->a * 6;
}
static void bump116_2(S116_2 *s, long d) {
    s->a = s->a + d;
}
static long classify116_2(int tag, long a, long b) {
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
static long accum116_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard116_2(long x) {
    return x + 5;
}

static long pick116_2_0(long a, long b) { return a > b ? a : b; }
static long pick116_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S116_3;

static S116_3 mk116_3(long a) {
    S116_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe116_3(const S116_3 *s) {
    return s->a + s->n0;
}
static long read116_3(const S116_3 *s) {
    return s->a * 4;
}
static void bump116_3(S116_3 *s, long d) {
    s->a = s->a + d;
}
static long classify116_3(int tag, long a, long b) {
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
static long accum116_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard116_3(long x) {
    return x + 1;
}

static long pick116_3_0(long a, long b) { return a > b ? a : b; }
long f116(long x) {
    long acc = x;
    acc += f001(x + 1);
    acc += f066(x + 2);
    acc += f077(x + 3);
    acc += f078(x + 4);
    S116_0 s0 = mk116_0(acc);
    bump116_0(&s0, 8);
    acc += probe116_0(&s0);
    acc += read116_0(&s0);
    acc += classify116_0(1, acc, acc);
    acc += accum116_0(3);
    acc += guard116_0(acc);
    acc += pick116_0_0(acc, acc + 4);
    acc += pick116_0_1(acc, acc + 5);
    S116_1 s1 = mk116_1(acc);
    bump116_1(&s1, 5);
    acc += probe116_1(&s1);
    acc += read116_1(&s1);
    acc += classify116_1(1, acc, acc);
    acc += accum116_1(7);
    acc += guard116_1(acc);
    acc += pick116_1_0(acc, acc + 5);
    S116_2 s2 = mk116_2(acc);
    bump116_2(&s2, 8);
    acc += probe116_2(&s2);
    acc += read116_2(&s2);
    acc += classify116_2(1, acc, acc);
    acc += accum116_2(5);
    acc += guard116_2(acc);
    acc += pick116_2_0(acc, acc + 6);
    acc += pick116_2_1(acc, acc + 1);
    S116_3 s3 = mk116_3(acc);
    bump116_3(&s3, 9);
    acc += probe116_3(&s3);
    acc += read116_3(&s3);
    acc += classify116_3(1, acc, acc);
    acc += accum116_3(9);
    acc += guard116_3(acc);
    acc += pick116_3_0(acc, acc + 1);
    return clampi(acc);
}
