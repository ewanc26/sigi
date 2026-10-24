#include <stdio.h>

int main(void) {
    long long a = -7;
    long long b = 2;
    printf("%g\n", (double)(a / b));
    a = 7;
    printf("%g\n", (double)(a / b));
    return 0;
}
