#include <stdio.h>
#include <stdint.h>

union Bits {
    uint32_t whole;
    uint8_t low;
    double wide;
};

static union Bits word;

int main(void) {
    printf("%g\n", (double)sizeof(union Bits));
    word.whole = 258;
    printf("%g\n", (double)word.low);
    printf("%g\n", (double)word.whole);
    word.low = 7;
    printf("%g\n", (double)word.whole);
    word.wide = 3;
    printf("%g\n", word.wide);
    printf("%g\n", (double)word.whole);
    return 0;
}
