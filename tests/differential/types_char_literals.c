#include <stdio.h>

int main(void) {
    printf("%g\n", (double)'A');
    printf("%g\n", (double)'z');
    printf("%g\n", (double)'\n');
    printf("%g\n", (double)'\t');
    printf("%g\n", (double)'\x41');
    printf("%g\n", (double)'\101');
    printf("%g\n", (double)'\\');
    printf("%g\n", (double)'\'');
    printf("%g\n", (double)'\0');
    putchar('a' + 1);
    printf("%g\n", (double)('\x7e' - 1));
    return 0;
}
