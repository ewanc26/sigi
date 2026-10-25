#include <stdio.h>

int main(void) {
    double n = 3;
    do {
        printf("%g\n", n);
        n = n - 1;
    } while (n != 0);

    do {
        printf("%g\n", 9.0);
    } while (0);

    n = 4;
    do {
        n = n - 1;
        if (n == 2) {
            continue;
        }
        printf("%g\n", n);
    } while (n != 0);
    return 0;
}
