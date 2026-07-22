/* GENERATED C mirror of reference module m091. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S91_0;

static S91_0 mk91_0(long a) {
    S91_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe91_0(const S91_0 *s) {
    return s->a + s->n0;
}
static long read91_0(const S91_0 *s) {
    return s->a * 3;
}
static void bump91_0(S91_0 *s, long d) {
    s->a = s->a + d;
}
static long classify91_0(int tag, long a, long b) {
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
static long accum91_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard91_0(long x) {
    return x + 2;
}

static long pick91_0_0(long a, long b) { return a > b ? a : b; }
static long pick91_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S91_1;

static S91_1 mk91_1(long a) {
    S91_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe91_1(const S91_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read91_1(const S91_1 *s) {
    return s->a * 3;
}
static void bump91_1(S91_1 *s, long d) {
    s->a = s->a + d;
}
static long classify91_1(int tag, long a, long b) {
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
static long accum91_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard91_1(long x) {
    return x + 2;
}

static long pick91_1_0(long a, long b) { return a > b ? a : b; }
static long pick91_1_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S91_2;

static S91_2 mk91_2(long a) {
    S91_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe91_2(const S91_2 *s) {
    return s->a + s->n0;
}
static long read91_2(const S91_2 *s) {
    return s->a * 2;
}
static void bump91_2(S91_2 *s, long d) {
    s->a = s->a + d;
}
static long classify91_2(int tag, long a, long b) {
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
static long accum91_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard91_2(long x) {
    return x + 8;
}

static long pick91_2_0(long a, long b) { return a > b ? a : b; }
static long pick91_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S91_3;

static S91_3 mk91_3(long a) {
    S91_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe91_3(const S91_3 *s) {
    return s->a + s->n0;
}
static long read91_3(const S91_3 *s) {
    return s->a * 4;
}
static void bump91_3(S91_3 *s, long d) {
    s->a = s->a + d;
}
static long classify91_3(int tag, long a, long b) {
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
static long accum91_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard91_3(long x) {
    return x + 6;
}

static long pick91_3_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S91_4;

static S91_4 mk91_4(long a) {
    S91_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe91_4(const S91_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read91_4(const S91_4 *s) {
    return s->a * 6;
}
static void bump91_4(S91_4 *s, long d) {
    s->a = s->a + d;
}
static long classify91_4(int tag, long a, long b) {
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
static long accum91_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard91_4(long x) {
    return x + 2;
}

static long pick91_4_0(long a, long b) { return a > b ? a : b; }
static long pick91_4_1(long a, long b) { return a > b ? a : b; }
long f091(long x) {
    long acc = x;
    acc += f014(x + 1);
    acc += f043(x + 2);
    acc += f060(x + 3);
    acc += f061(x + 4);
    S91_0 s0 = mk91_0(acc);
    bump91_0(&s0, 7);
    acc += probe91_0(&s0);
    acc += read91_0(&s0);
    acc += classify91_0(1, acc, acc);
    acc += accum91_0(7);
    acc += guard91_0(acc);
    acc += pick91_0_0(acc, acc + 4);
    acc += pick91_0_1(acc, acc + 5);
    S91_1 s1 = mk91_1(acc);
    bump91_1(&s1, 5);
    acc += probe91_1(&s1);
    acc += read91_1(&s1);
    acc += classify91_1(1, acc, acc);
    acc += accum91_1(4);
    acc += guard91_1(acc);
    acc += pick91_1_0(acc, acc + 8);
    acc += pick91_1_1(acc, acc + 1);
    S91_2 s2 = mk91_2(acc);
    bump91_2(&s2, 1);
    acc += probe91_2(&s2);
    acc += read91_2(&s2);
    acc += classify91_2(1, acc, acc);
    acc += accum91_2(4);
    acc += guard91_2(acc);
    acc += pick91_2_0(acc, acc + 1);
    acc += pick91_2_1(acc, acc + 7);
    S91_3 s3 = mk91_3(acc);
    bump91_3(&s3, 6);
    acc += probe91_3(&s3);
    acc += read91_3(&s3);
    acc += classify91_3(1, acc, acc);
    acc += accum91_3(4);
    acc += guard91_3(acc);
    acc += pick91_3_0(acc, acc + 9);
    S91_4 s4 = mk91_4(acc);
    bump91_4(&s4, 1);
    acc += probe91_4(&s4);
    acc += read91_4(&s4);
    acc += classify91_4(1, acc, acc);
    acc += accum91_4(3);
    acc += guard91_4(acc);
    acc += pick91_4_0(acc, acc + 5);
    acc += pick91_4_1(acc, acc + 7);
    return clampi(acc);
}
