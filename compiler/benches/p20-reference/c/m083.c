/* GENERATED C mirror of reference module m083. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S83_0;

static S83_0 mk83_0(long a) {
    S83_0 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe83_0(const S83_0 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read83_0(const S83_0 *s) {
    return s->a * 6;
}
static void bump83_0(S83_0 *s, long d) {
    s->a = s->a + d;
}
static long classify83_0(int tag, long a, long b) {
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
static long accum83_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard83_0(long x) {
    return x + 6;
}

static long pick83_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S83_1;

static S83_1 mk83_1(long a) {
    S83_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe83_1(const S83_1 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read83_1(const S83_1 *s) {
    return s->a * 6;
}
static void bump83_1(S83_1 *s, long d) {
    s->a = s->a + d;
}
static long classify83_1(int tag, long a, long b) {
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
static long accum83_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard83_1(long x) {
    return x + 9;
}

static long pick83_1_0(long a, long b) { return a > b ? a : b; }
static long pick83_1_1(long a, long b) { return a > b ? a : b; }
static long pick83_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S83_2;

static S83_2 mk83_2(long a) {
    S83_2 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe83_2(const S83_2 *s) {
    return s->a + s->n0;
}
static long read83_2(const S83_2 *s) {
    return s->a * 2;
}
static void bump83_2(S83_2 *s, long d) {
    s->a = s->a + d;
}
static long classify83_2(int tag, long a, long b) {
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
static long accum83_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard83_2(long x) {
    return x + 7;
}

static long pick83_2_0(long a, long b) { return a > b ? a : b; }
static long pick83_2_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S83_3;

static S83_3 mk83_3(long a) {
    S83_3 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe83_3(const S83_3 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read83_3(const S83_3 *s) {
    return s->a * 7;
}
static void bump83_3(S83_3 *s, long d) {
    s->a = s->a + d;
}
static long classify83_3(int tag, long a, long b) {
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
static long accum83_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard83_3(long x) {
    return x + 9;
}

static long pick83_3_0(long a, long b) { return a > b ? a : b; }
static long pick83_3_1(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S83_4;

static S83_4 mk83_4(long a) {
    S83_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe83_4(const S83_4 *s) {
    return s->a + s->n0 + s->n1;
}
static long read83_4(const S83_4 *s) {
    return s->a * 6;
}
static void bump83_4(S83_4 *s, long d) {
    s->a = s->a + d;
}
static long classify83_4(int tag, long a, long b) {
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
static long accum83_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard83_4(long x) {
    return x + 7;
}

static long pick83_4_0(long a, long b) { return a > b ? a : b; }
long f083(long x) {
    long acc = x;
    acc += f005(x + 1);
    acc += f019(x + 2);
    acc += f040(x + 3);
    acc += f061(x + 4);
    S83_0 s0 = mk83_0(acc);
    bump83_0(&s0, 9);
    acc += probe83_0(&s0);
    acc += read83_0(&s0);
    acc += classify83_0(1, acc, acc);
    acc += accum83_0(7);
    acc += guard83_0(acc);
    acc += pick83_0_0(acc, acc + 4);
    S83_1 s1 = mk83_1(acc);
    bump83_1(&s1, 7);
    acc += probe83_1(&s1);
    acc += read83_1(&s1);
    acc += classify83_1(1, acc, acc);
    acc += accum83_1(4);
    acc += guard83_1(acc);
    acc += pick83_1_0(acc, acc + 4);
    acc += pick83_1_1(acc, acc + 5);
    acc += pick83_1_2(acc, acc + 5);
    S83_2 s2 = mk83_2(acc);
    bump83_2(&s2, 1);
    acc += probe83_2(&s2);
    acc += read83_2(&s2);
    acc += classify83_2(1, acc, acc);
    acc += accum83_2(5);
    acc += guard83_2(acc);
    acc += pick83_2_0(acc, acc + 1);
    acc += pick83_2_1(acc, acc + 3);
    S83_3 s3 = mk83_3(acc);
    bump83_3(&s3, 9);
    acc += probe83_3(&s3);
    acc += read83_3(&s3);
    acc += classify83_3(1, acc, acc);
    acc += accum83_3(6);
    acc += guard83_3(acc);
    acc += pick83_3_0(acc, acc + 8);
    acc += pick83_3_1(acc, acc + 4);
    S83_4 s4 = mk83_4(acc);
    bump83_4(&s4, 7);
    acc += probe83_4(&s4);
    acc += read83_4(&s4);
    acc += classify83_4(1, acc, acc);
    acc += accum83_4(8);
    acc += guard83_4(acc);
    acc += pick83_4_0(acc, acc + 5);
    return clampi(acc);
}
