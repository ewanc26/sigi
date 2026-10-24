#include <stdio.h>

int main(void) {
    double truthy = 1;
    double falsy = 0;
    if (truthy != 0) {
        printf("%g\n", 10.0);
    } else {
        printf("%g\n", 20.0);
    }
    if (falsy != 0) {
        printf("%g\n", 10.0);
    } else {
        printf("%g\n", 20.0);
    }
    return 0;
}
