#include <stdio.h>

int main(void) {
    double items[2] = {0};
    size_t index = 2;
    if (index >= sizeof items / sizeof items[0]) {
        return 1;
    }
    printf("%g\n", items[index]);
    return 0;
}
