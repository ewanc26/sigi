#include <stdio.h>

enum color { RED, GREEN, BLUE = 5, NEXT, LAST };

int main(void) {
    printf("%d\n", RED);
    printf("%d\n", GREEN);
    printf("%d\n", BLUE);
    printf("%d\n", NEXT);
    printf("%d\n", LAST);
    if (LAST == 7) {
        printf("%d\n", 100);
    } else {
        printf("%d\n", 200);
    }
    return 0;
}
