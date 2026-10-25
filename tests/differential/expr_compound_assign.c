#include <stdio.h>

int main(void) {
    double x = 5;
    x += 3;
    printf("%g\n", x);
    x *= 2;
    printf("%g\n", x);
    x -= 1;
    printf("%g\n", x);
    x = (double)((long long)x << 2);
    printf("%g\n", x);
    return 0;
}
