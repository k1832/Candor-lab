/* GENERATED C mirror of reference module m148. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S148_0;

static S148_0 mk148_0(long a) {
    S148_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe148_0(const S148_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read148_0(const S148_0 *s) {
    return s->a * 4;
}
static void bump148_0(S148_0 *s, long d) {
    s->a = s->a + d;
}
static long classify148_0(int tag, long a, long b) {
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
static long accum148_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard148_0(long x) {
    return x + 4;
}

static long pick148_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S148_1;

static S148_1 mk148_1(long a) {
    S148_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe148_1(const S148_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read148_1(const S148_1 *s) {
    return s->a * 5;
}
static void bump148_1(S148_1 *s, long d) {
    s->a = s->a + d;
}
static long classify148_1(int tag, long a, long b) {
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
static long accum148_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard148_1(long x) {
    return x + 4;
}

static long pick148_1_0(long a, long b) { return a > b ? a : b; }
static long pick148_1_1(long a, long b) { return a > b ? a : b; }
static long pick148_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S148_2;

static S148_2 mk148_2(long a) {
    S148_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe148_2(const S148_2 *s) {
    return s->a + s->n0;
}
static long read148_2(const S148_2 *s) {
    return s->a * 5;
}
static void bump148_2(S148_2 *s, long d) {
    s->a = s->a + d;
}
static long classify148_2(int tag, long a, long b) {
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
static long accum148_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard148_2(long x) {
    return x + 1;
}

static long pick148_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S148_3;

static S148_3 mk148_3(long a) {
    S148_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe148_3(const S148_3 *s) {
    return s->a + s->n0;
}
static long read148_3(const S148_3 *s) {
    return s->a * 3;
}
static void bump148_3(S148_3 *s, long d) {
    s->a = s->a + d;
}
static long classify148_3(int tag, long a, long b) {
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
static long accum148_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard148_3(long x) {
    return x + 5;
}

static long pick148_3_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S148_4;

static S148_4 mk148_4(long a) {
    S148_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe148_4(const S148_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read148_4(const S148_4 *s) {
    return s->a * 5;
}
static void bump148_4(S148_4 *s, long d) {
    s->a = s->a + d;
}
static long classify148_4(int tag, long a, long b) {
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
static long accum148_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard148_4(long x) {
    return x + 3;
}

static long pick148_4_0(long a, long b) { return a > b ? a : b; }
static long pick148_4_1(long a, long b) { return a > b ? a : b; }
static long pick148_4_2(long a, long b) { return a > b ? a : b; }
long f148(long x) {
    long acc = x;
    acc += f119(x + 1);
    S148_0 s0 = mk148_0(acc);
    bump148_0(&s0, 7);
    acc += probe148_0(&s0);
    acc += read148_0(&s0);
    acc += classify148_0(1, acc, acc);
    acc += accum148_0(3);
    acc += guard148_0(acc);
    acc += pick148_0_0(acc, acc + 1);
    S148_1 s1 = mk148_1(acc);
    bump148_1(&s1, 4);
    acc += probe148_1(&s1);
    acc += read148_1(&s1);
    acc += classify148_1(1, acc, acc);
    acc += accum148_1(6);
    acc += guard148_1(acc);
    acc += pick148_1_0(acc, acc + 9);
    acc += pick148_1_1(acc, acc + 8);
    acc += pick148_1_2(acc, acc + 6);
    S148_2 s2 = mk148_2(acc);
    bump148_2(&s2, 5);
    acc += probe148_2(&s2);
    acc += read148_2(&s2);
    acc += classify148_2(1, acc, acc);
    acc += accum148_2(5);
    acc += guard148_2(acc);
    acc += pick148_2_0(acc, acc + 1);
    S148_3 s3 = mk148_3(acc);
    bump148_3(&s3, 8);
    acc += probe148_3(&s3);
    acc += read148_3(&s3);
    acc += classify148_3(1, acc, acc);
    acc += accum148_3(8);
    acc += guard148_3(acc);
    acc += pick148_3_0(acc, acc + 1);
    S148_4 s4 = mk148_4(acc);
    bump148_4(&s4, 3);
    acc += probe148_4(&s4);
    acc += read148_4(&s4);
    acc += classify148_4(1, acc, acc);
    acc += accum148_4(6);
    acc += guard148_4(acc);
    acc += pick148_4_0(acc, acc + 2);
    acc += pick148_4_1(acc, acc + 4);
    acc += pick148_4_2(acc, acc + 5);
    return clampi(acc);
}
