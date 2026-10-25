#include <stdio.h>

static void one(void) { printf("%g\n", 1.0); }
static void two(void) { printf("%g\n", 2.0); }
static void seven(void) { printf("%g\n", 77.0); }

int main(void) {
    void (*fp)(void);
    fp = one;
    fp();
    fp = two;
    fp();
    fp = seven;
    fp();
    fp = one;
    fp();
    return 0;
}
