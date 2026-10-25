#include <stdio.h>
#include <stdint.h>

typedef uint8_t byte;

int main(void) {
    printf("%u\n", (byte)300u);
    printf("%zu\n", sizeof((byte)7));
    printf("%d\n", (byte)200u + (byte)100u);
    return 0;
}
