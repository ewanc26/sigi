#include <stdio.h>

static void countdown(double n) {
    printf("%g\n", n);
    if (n > 0) {
        countdown(n - 1);
    }
}

int main(void) {
    countdown(3);
    return 0;
}
