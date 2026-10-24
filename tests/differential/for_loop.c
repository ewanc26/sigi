#include <stdio.h>

int main(void) {
    double total = 0;
    for (int i = 0; i < 4; i++) {
        total += 1;
    }
    printf("%g\n", total);

    double doubled = 1;
    for (int i = 0; i < 3; i++) {
        doubled = doubled + doubled;
    }
    printf("%g\n", doubled);
    return 0;
}
