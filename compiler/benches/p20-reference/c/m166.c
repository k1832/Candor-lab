/* GENERATED C mirror of reference module m166. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S166_0;

static S166_0 mk166_0(long a) {
    S166_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe166_0(const S166_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read166_0(const S166_0 *s) {
    return s->a * 6;
}
static void bump166_0(S166_0 *s, long d) {
    s->a = s->a + d;
}
static long classify166_0(int tag, long a, long b) {
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
static long accum166_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard166_0(long x) {
    return x + 2;
}

static long pick166_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S166_1;

static S166_1 mk166_1(long a) {
    S166_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe166_1(const S166_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read166_1(const S166_1 *s) {
    return s->a * 2;
}
static void bump166_1(S166_1 *s, long d) {
    s->a = s->a + d;
}
static long classify166_1(int tag, long a, long b) {
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
static long accum166_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard166_1(long x) {
    return x + 5;
}

static long pick166_1_0(long a, long b) { return a > b ? a : b; }
static long pick166_1_1(long a, long b) { return a > b ? a : b; }
static long pick166_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S166_2;

static S166_2 mk166_2(long a) {
    S166_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe166_2(const S166_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read166_2(const S166_2 *s) {
    return s->a * 3;
}
static void bump166_2(S166_2 *s, long d) {
    s->a = s->a + d;
}
static long classify166_2(int tag, long a, long b) {
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
static long accum166_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard166_2(long x) {
    return x + 6;
}

static long pick166_2_0(long a, long b) { return a > b ? a : b; }
static long pick166_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S166_3;

static S166_3 mk166_3(long a) {
    S166_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe166_3(const S166_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read166_3(const S166_3 *s) {
    return s->a * 6;
}
static void bump166_3(S166_3 *s, long d) {
    s->a = s->a + d;
}
static long classify166_3(int tag, long a, long b) {
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
static long accum166_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard166_3(long x) {
    return x + 2;
}

static long pick166_3_0(long a, long b) { return a > b ? a : b; }
static long pick166_3_1(long a, long b) { return a > b ? a : b; }
static long pick166_3_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S166_4;

static S166_4 mk166_4(long a) {
    S166_4 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe166_4(const S166_4 *s) {
    return s->a + s->n0;
}
static long read166_4(const S166_4 *s) {
    return s->a * 4;
}
static void bump166_4(S166_4 *s, long d) {
    s->a = s->a + d;
}
static long classify166_4(int tag, long a, long b) {
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
static long accum166_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard166_4(long x) {
    return x + 1;
}

static long pick166_4_0(long a, long b) { return a > b ? a : b; }
static long pick166_4_1(long a, long b) { return a > b ? a : b; }
long f166(long x) {
    long acc = x;
    acc += f123(x + 1);
    S166_0 s0 = mk166_0(acc);
    bump166_0(&s0, 8);
    acc += probe166_0(&s0);
    acc += read166_0(&s0);
    acc += classify166_0(1, acc, acc);
    acc += accum166_0(3);
    acc += guard166_0(acc);
    acc += pick166_0_0(acc, acc + 8);
    S166_1 s1 = mk166_1(acc);
    bump166_1(&s1, 3);
    acc += probe166_1(&s1);
    acc += read166_1(&s1);
    acc += classify166_1(1, acc, acc);
    acc += accum166_1(7);
    acc += guard166_1(acc);
    acc += pick166_1_0(acc, acc + 6);
    acc += pick166_1_1(acc, acc + 5);
    acc += pick166_1_2(acc, acc + 8);
    S166_2 s2 = mk166_2(acc);
    bump166_2(&s2, 8);
    acc += probe166_2(&s2);
    acc += read166_2(&s2);
    acc += classify166_2(1, acc, acc);
    acc += accum166_2(9);
    acc += guard166_2(acc);
    acc += pick166_2_0(acc, acc + 8);
    acc += pick166_2_1(acc, acc + 6);
    S166_3 s3 = mk166_3(acc);
    bump166_3(&s3, 7);
    acc += probe166_3(&s3);
    acc += read166_3(&s3);
    acc += classify166_3(1, acc, acc);
    acc += accum166_3(7);
    acc += guard166_3(acc);
    acc += pick166_3_0(acc, acc + 3);
    acc += pick166_3_1(acc, acc + 3);
    acc += pick166_3_2(acc, acc + 6);
    S166_4 s4 = mk166_4(acc);
    bump166_4(&s4, 9);
    acc += probe166_4(&s4);
    acc += read166_4(&s4);
    acc += classify166_4(1, acc, acc);
    acc += accum166_4(6);
    acc += guard166_4(acc);
    acc += pick166_4_0(acc, acc + 8);
    acc += pick166_4_1(acc, acc + 5);
    return clampi(acc);
}
