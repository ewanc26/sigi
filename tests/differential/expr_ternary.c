#include <stdio.h>

int main(void) {
    double yes = 1;
    double no = 0;
    printf("%g\n", yes != 0 ? 10.0 : 20.0);
    printf("%g\n", no != 0 ? 10.0 : 20.0);
    return 0;
}
