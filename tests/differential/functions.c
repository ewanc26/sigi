#include <stdio.h>

static double square(double x) {
    return x * x;
}

static double add_hundred(double x) {
    return x + 100.0;
}

int main(void) {
    printf("%g\n", square(6.0));
    printf("%g\n", add_hundred(5.0));
    return 0;
}
