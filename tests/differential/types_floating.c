#include <math.h>
#include <stdint.h>
#include <stdio.h>

int main(void) {
    printf("%Lg\n", 1.0L / 3.0L);
    printf("%Lg\n", fmodl(-7.0L, 3.0L));
    printf("%Lg\n", (long double)(int32_t)7 / 2.0L);
    printf("%Lg\n", (long double)2.5 * (long double)(float)1.5);
    printf("%Lg\n", (long double)(float)16777217);
    printf("%d\n", (int32_t)(long double)-3.9);
    printf("%zu\n", sizeof(long double));
    float items[4] = {0};
    items[1] = 1.0f / 3.0f;
    printf("%g\n", (double)items[1]);
    return 0;
}
