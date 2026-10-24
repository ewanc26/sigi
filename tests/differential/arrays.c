#include <stdio.h>

int main(void) {
    double items[5] = {0};
    items[2] = 7;
    printf("%g\n", items[2]);
    printf("%g\n", (double)(sizeof items / sizeof items[0]));
    return 0;
}
