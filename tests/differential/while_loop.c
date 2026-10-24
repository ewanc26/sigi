#include <stdio.h>

int main(void) {
    int counter = 3;
    while (counter != 0) {
        printf("%g\n", (double)counter);
        counter -= 1;
    }
    return 0;
}
