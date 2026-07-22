/* GENERATED C mirror of reference module m175. */
#include "proto.h"

typedef struct {
    long a;
    long n0;
    int flag;
} S175_0;

static S175_0 mk175_0(long a) {
    S175_0 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe175_0(const S175_0 *s) {
    return s->a + s->n0;
}
static long read175_0(const S175_0 *s) {
    return s->a * 3;
}
static void bump175_0(S175_0 *s, long d) {
    s->a = s->a + d;
}
static long classify175_0(int tag, long a, long b) {
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
static long accum175_0(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 4;
    }
    return acc;
}
static long guard175_0(long x) {
    return x + 9;
}

static long pick175_0_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    int flag;
} S175_1;

static S175_1 mk175_1(long a) {
    S175_1 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.flag = 1;
    return s;
}
static long probe175_1(const S175_1 *s) {
    return s->a + s->n0 + s->n1;
}
static long read175_1(const S175_1 *s) {
    return s->a * 5;
}
static void bump175_1(S175_1 *s, long d) {
    s->a = s->a + d;
}
static long classify175_1(int tag, long a, long b) {
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
static long accum175_1(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard175_1(long x) {
    return x + 9;
}

static long pick175_1_0(long a, long b) { return a > b ? a : b; }
static long pick175_1_1(long a, long b) { return a > b ? a : b; }
static long pick175_1_2(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S175_2;

static S175_2 mk175_2(long a) {
    S175_2 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe175_2(const S175_2 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read175_2(const S175_2 *s) {
    return s->a * 5;
}
static void bump175_2(S175_2 *s, long d) {
    s->a = s->a + d;
}
static long classify175_2(int tag, long a, long b) {
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
static long accum175_2(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 3;
    }
    return acc;
}
static long guard175_2(long x) {
    return x + 3;
}

static long pick175_2_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    int flag;
} S175_3;

static S175_3 mk175_3(long a) {
    S175_3 s;
    s.a = a;
    s.n0 = 0;
    s.flag = 1;
    return s;
}
static long probe175_3(const S175_3 *s) {
    return s->a + s->n0;
}
static long read175_3(const S175_3 *s) {
    return s->a * 3;
}
static void bump175_3(S175_3 *s, long d) {
    s->a = s->a + d;
}
static long classify175_3(int tag, long a, long b) {
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
static long accum175_3(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 5;
    }
    return acc;
}
static long guard175_3(long x) {
    return x + 4;
}

static long pick175_3_0(long a, long b) { return a > b ? a : b; }
typedef struct {
    long a;
    long n0;
    long n1;
    long n2;
    int flag;
} S175_4;

static S175_4 mk175_4(long a) {
    S175_4 s;
    s.a = a;
    s.n0 = 0;
    s.n1 = 0;
    s.n2 = 0;
    s.flag = 1;
    return s;
}
static long probe175_4(const S175_4 *s) {
    return s->a + s->n0 + s->n1 + s->n2;
}
static long read175_4(const S175_4 *s) {
    return s->a * 6;
}
static void bump175_4(S175_4 *s, long d) {
    s->a = s->a + d;
}
static long classify175_4(int tag, long a, long b) {
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
static long accum175_4(long n) {
    long acc = 0;
    long i = 0;
    for (i = 0; i < n; i++) {
        acc += i * 2;
    }
    return acc;
}
static long guard175_4(long x) {
    return x + 8;
}

static long pick175_4_0(long a, long b) { return a > b ? a : b; }
static long pick175_4_1(long a, long b) { return a > b ? a : b; }
long f175(long x) {
    long acc = x;
    acc += f075(x + 1);
    acc += f164(x + 2);
    S175_0 s0 = mk175_0(acc);
    bump175_0(&s0, 6);
    acc += probe175_0(&s0);
    acc += read175_0(&s0);
    acc += classify175_0(1, acc, acc);
    acc += accum175_0(3);
    acc += guard175_0(acc);
    acc += pick175_0_0(acc, acc + 9);
    S175_1 s1 = mk175_1(acc);
    bump175_1(&s1, 3);
    acc += probe175_1(&s1);
    acc += read175_1(&s1);
    acc += classify175_1(1, acc, acc);
    acc += accum175_1(8);
    acc += guard175_1(acc);
    acc += pick175_1_0(acc, acc + 1);
    acc += pick175_1_1(acc, acc + 9);
    acc += pick175_1_2(acc, acc + 3);
    S175_2 s2 = mk175_2(acc);
    bump175_2(&s2, 4);
    acc += probe175_2(&s2);
    acc += read175_2(&s2);
    acc += classify175_2(1, acc, acc);
    acc += accum175_2(4);
    acc += guard175_2(acc);
    acc += pick175_2_0(acc, acc + 2);
    S175_3 s3 = mk175_3(acc);
    bump175_3(&s3, 2);
    acc += probe175_3(&s3);
    acc += read175_3(&s3);
    acc += classify175_3(1, acc, acc);
    acc += accum175_3(9);
    acc += guard175_3(acc);
    acc += pick175_3_0(acc, acc + 4);
    S175_4 s4 = mk175_4(acc);
    bump175_4(&s4, 5);
    acc += probe175_4(&s4);
    acc += read175_4(&s4);
    acc += classify175_4(1, acc, acc);
    acc += accum175_4(9);
    acc += guard175_4(acc);
    acc += pick175_4_0(acc, acc + 6);
    acc += pick175_4_1(acc, acc + 9);
    return clampi(acc);
}
