#include <stdio.h>

int main(void) {
    printf("%g\n", (double)((2 != 0) && (3 != 0)));
    printf("%g\n", (double)((0 != 0) && (3 != 0)));
    printf("%g\n", (double)((0 != 0) || (0 != 0)));
    printf("%g\n", (double)((4 != 0) || (0 != 0)));
    printf("%g\n", (double)(!5));
    return 0;
}
