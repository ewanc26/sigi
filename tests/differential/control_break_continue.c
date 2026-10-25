#include <stdio.h>

int main(void) {
    double i = 0;
    for (int k = 0; k < 10; k++) {
        i = i + 1;
        if (i == 3) {
            continue;
        }
        if (i == 6) {
            break;
        }
        printf("%g\n", i);
    }

    double n = 5;
    while (n != 0) {
        n = n - 1;
        if (n == 3) {
            continue;
        }
        printf("%g\n", n);
    }

    n = 5;
    while (n != 0) {
        n = n - 1;
        if (n == 2) {
            break;
        }
        printf("%g\n", n);
    }
    return 0;
}
