/* GENERATED C mirror of reference module m076. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S76_0;

static S76_0 mk76_0(long a) {
    S76_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe76_0(const S76_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read76_0(const S76_0 *s) {
    return s->a * 7;
}
static void bump76_0(S76_0 *s, long d) {
    s->a = s->a + d;
}
static long classify76_0(int tag, long a, long b) {
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
static long accum76_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard76_0(long x) {
    return x + 5;
}

static long pick76_0_0(long a, long b) { return a > b ? a : b; }
static long pick76_0_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S76_1;

static S76_1 mk76_1(long a) {
    S76_1 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe76_1(const S76_1 *s) {
    return s->a + s->n0;
}
static long read76_1(const S76_1 *s) {
    return s->a * 2;
}
static void bump76_1(S76_1 *s, long d) {
    s->a = s->a + d;
}
static long classify76_1(int tag, long a, long b) {
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
static long accum76_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard76_1(long x) {
    return x + 1;
}

static long pick76_1_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S76_2;

static S76_2 mk76_2(long a) {
    S76_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe76_2(const S76_2 *s) {
    return s->a + s->n0 + s->n1;
}
static long read76_2(const S76_2 *s) {
    return s->a * 5;
}
static void bump76_2(S76_2 *s, long d) {
    s->a = s->a + d;
}
static long classify76_2(int tag, long a, long b) {
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
static long accum76_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard76_2(long x) {
    return x + 7;
}

static long pick76_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S76_3;

static S76_3 mk76_3(long a) {
    S76_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe76_3(const S76_3 *s) {
    return s->a + s->n0;
}
static long read76_3(const S76_3 *s) {
    return s->a * 7;
}
static void bump76_3(S76_3 *s, long d) {
    s->a = s->a + d;
}
static long classify76_3(int tag, long a, long b) {
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
static long accum76_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard76_3(long x) {
    return x + 2;
}

static long pick76_3_0(long a, long b) { return a > b ? a : b; }
static long pick76_3_1(long a, long b) { return a > b ? a : b; }
long f076(long x) {
    long acc = x;
    acc += f018(x + 1);
    acc += f029(x + 2);
    acc += f035(x + 3);
    acc += f041(x + 4);
    S76_0 s0 = mk76_0(acc);
    bump76_0(&s0, 2);
    acc += probe76_0(&s0);
    acc += read76_0(&s0);
    acc += classify76_0(1, acc, acc);
    acc += accum76_0(7);
    acc += guard76_0(acc);
    acc += pick76_0_0(acc, acc + 3);
    acc += pick76_0_1(acc, acc + 8);
    S76_1 s1 = mk76_1(acc);
    bump76_1(&s1, 4);
    acc += probe76_1(&s1);
    acc += read76_1(&s1);
    acc += classify76_1(1, acc, acc);
    acc += accum76_1(5);
    acc += guard76_1(acc);
    acc += pick76_1_0(acc, acc + 6);
    S76_2 s2 = mk76_2(acc);
    bump76_2(&s2, 2);
    acc += probe76_2(&s2);
    acc += read76_2(&s2);
    acc += classify76_2(1, acc, acc);
    acc += accum76_2(8);
    acc += guard76_2(acc);
    acc += pick76_2_0(acc, acc + 2);
    S76_3 s3 = mk76_3(acc);
    bump76_3(&s3, 2);
    acc += probe76_3(&s3);
    acc += read76_3(&s3);
    acc += classify76_3(1, acc, acc);
    acc += accum76_3(7);
    acc += guard76_3(acc);
    acc += pick76_3_0(acc, acc + 6);
    acc += pick76_3_1(acc, acc + 1);
    return clampi(acc);
}
