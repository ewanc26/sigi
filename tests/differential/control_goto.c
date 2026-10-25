#include <stdio.h>

int main(void) {
    double i = 0;
again:
    i = i + 1;
    printf("%g\n", i);
    if (i - 3 != 0) {
        goto again;
    }
    if (1.0 != 0) {
        goto skip;
    }
    printf("%g\n", 99.0);
skip:
    printf("%g\n", 42.0);
    return 0;
}
