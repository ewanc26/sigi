#include <stdio.h>
#include <stdint.h>

static int32_t m[3][4];
static double v[3];

int main(void) {
    m[1][2] = 7;
    m[2][3] = 5;
    printf("%g\n", (double)m[0][0]);
    printf("%g\n", (double)m[1][2]);
    printf("%g\n", (double)m[2][3]);
    v[2] = 2.5;
    printf("%g\n", v[2]);
    return 0;
}
