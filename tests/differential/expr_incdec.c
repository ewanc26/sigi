#include <stdio.h>

int main(void) {
    double x = 5;
    x = x + 1;
    printf("%g\n", x);
    printf("%g\n", x);
    x = x + 1;
    printf("%g\n", x);
    x = x - 1;
    printf("%g\n", x);
    printf("%g\n", x);
    x = x - 1;
    printf("%g\n", x);
    return 0;
}
