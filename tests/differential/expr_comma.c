#include <stdio.h>

int main(void) {
    double a = 0;
    printf("%g\n", (a = 1, a + 1.0));
    printf("%g\n", (a = 3, a + 1.0));
    return 0;
}
