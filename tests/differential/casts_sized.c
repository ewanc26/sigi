#include <stdio.h>
#include <stdint.h>

int main(void) {
    printf("%u\n", (uint8_t)300u);
    printf("%u\n", (uint16_t)-1l);
    printf("%d\n", (int8_t)200u);
    printf("%d\n", (int32_t)3.9);
    printf("%d\n", (int32_t)-3.9);
    printf("%d\n", (int32_t)(uint16_t)65537u);
    printf("%d\n", (int8_t)255u);
    printf("%g\n", (double)(float)2.5);
    printf("%zu\n", sizeof(int16_t));
    printf("%zu\n", sizeof(double));
    printf("%zu\n", sizeof(uint32_t));
    printf("%zu\n", sizeof(7u));
    return 0;
}
