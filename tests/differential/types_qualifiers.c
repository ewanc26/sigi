#include <stdio.h>
#include <stdint.h>

/* Storing through a pointer to the const object is rejected at run time
 * by sigi (exit 1); C cannot express that store, so the twin stops there. */
static int32_t count = 3;
static const double limit = 2.5;

int main(void) {
    printf("%g\n", (double)count);
    count = 4;
    printf("%g\n", (double)count);
    printf("%g\n", limit);
    printf("%g\n", (double)count + limit);
    return 1;
}
