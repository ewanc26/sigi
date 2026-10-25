#include <stdio.h>

int main(void) {
    printf("%g\n", (double)0xFF);
    printf("%g\n", (double)0X10);
    printf("%g\n", (double)11);
    printf("%g\n", (double)3);
    printf("%g\n", (double)017);
    printf("%g\n", (double)0);
    printf("%g\n", (double)-0x10);
    printf("%g\n", (double)10UL);
    printf("%g\n", (double)42ll);
    printf("%g\n", 007.5);
    printf("%g\n", (double)(7 + 0x1F));
    return 0;
}
