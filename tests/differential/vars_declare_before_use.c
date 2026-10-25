#include <stdio.h>

static double slots[3];

static void fill(void) {
    slots[0] = 1.5;
    slots[1] = 2.5;
    slots[2] = 4.25;
}

int main(void) {
    fill();
    for (int i = 0; i < 3; i++) {
        printf("%g\n", slots[i]);
    }
    return 0;
}
