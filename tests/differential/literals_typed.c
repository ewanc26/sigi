#include <stdio.h>

int main(void) {
    unsigned int big = 4000000000u;
    printf("%u\n", 4000000000u);
    printf("%llu\n", 18446744073709551615ull);
    printf("%lld\n", -5ll);
    printf("%lld\n", 9223372036854775807ll);
    printf("%u\n", 5u - 7u);
    printf("%d\n", 5u < 7u);
    printf("%lld\n", -1ll + 1u);
    printf("%lld\n", -7ll / 2ll);
    printf("%u\n", 7u / 2u);
    printf("%u\n", big);
    return 0;
}
