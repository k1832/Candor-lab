#include "proto.h"

long clampi(long x) {
    if (x > 1000000) return 1000000;
    if (x < -1000000) return -1000000;
    return x;
}
