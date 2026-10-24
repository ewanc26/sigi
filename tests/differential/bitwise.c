#include <stdio.h>

int main(void) {
    long long a = 12;
    long long b = 10;
    printf("%g\n", (double)(a & b));
    printf("%g\n", (double)(a | b));
    printf("%g\n", (double)(a ^ b));
    printf("%g\n", (double)(~5LL));
    printf("%g\n", (double)(1LL << 3));
    printf("%g\n", (double)(16LL >> 2));
    return 0;
}
