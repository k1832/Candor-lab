#include "proto.h"

int main(void) {
    long acc = 0;
    acc += f000(1);
    acc += f009(2);
    acc += f026(3);
    acc += f043(4);
    acc += f060(5);
    acc += f077(6);
    acc += f094(7);
    acc += f111(1);
    acc += f128(2);
    acc += f145(3);
    acc += f162(4);
    acc += f179(5);
    acc += f196(6);
    acc += f197(7);
    acc += f198(1);
    acc += f199(2);
    return (int)(acc & 0xff);
}
