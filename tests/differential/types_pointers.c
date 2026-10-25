#include <stdio.h>

static int x;
static int *p;
static int vars[1];

int main(void) {
    x = 5;
    printf("%g\n", (double)x);
    printf("%g\n", (double)*&x);
    x = 7;
    printf("%g\n", (double)x);
    printf("%g\n", (double)(&x == &x));
    p = &x;
    *p = 9;
    printf("%g\n", (double)x);
    vars[0] = 4;
    printf("%g\n", (double)vars[0]);
    printf("%g\n", (double)(&vars[0] == &x));
    return 0;
}
