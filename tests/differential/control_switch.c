#include <stdio.h>

int main(void) {
    switch (2) {
    case 1:
        printf("%g\n", 100.0);
        break;
    case 2:
        printf("%g\n", 200.0);
        break;
    default:
        printf("%g\n", 999.0);
        break;
    }

    switch (7) {
    case 1:
        printf("%g\n", 100.0);
        break;
    default:
        printf("%g\n", 999.0);
        break;
    }

    switch (1) {
    case 1:
        printf("%g\n", 10.0);
        [[fallthrough]];
    case 2:
        printf("%g\n", 20.0);
        break;
    default:
        printf("%g\n", 30.0);
    }
    return 0;
}
