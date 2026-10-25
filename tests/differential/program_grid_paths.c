#include <stdio.h>

static double blocked[25];

static double paths(int idx) {
    if (blocked[idx] != 0) {
        return 0;
    }
    if (idx == 6) {
        return 1;
    }
    return paths(idx - 5) + paths(idx - 1);
}

int main(void) {
    blocked[0] = 1;
    blocked[1] = 1;
    blocked[2] = 1;
    blocked[3] = 1;
    blocked[4] = 1;
    blocked[5] = 1;
    blocked[10] = 1;
    blocked[15] = 1;
    blocked[20] = 1;
    blocked[12] = 1;
    blocked[17] = 1;
    printf("%g\n", paths(24));
    printf("%g\n", paths(19));
    printf("%g\n", paths(13));
    printf("%g\n", paths(6));
    return 0;
}
