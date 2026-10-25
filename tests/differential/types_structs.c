#include <stdio.h>
#include <stdint.h>

struct Pt {
    double x;
    int32_t y;
};

struct Seg {
    struct Pt a;
    struct Pt b;
};

static struct Pt origin;
static struct Seg line;

int main(void) {
    printf("%g\n", (double)sizeof(struct Pt));
    printf("%g\n", (double)sizeof(struct Seg));
    origin.x = 2.5;
    origin.y = 7;
    printf("%g\n", origin.x);
    printf("%g\n", (double)origin.y);
    line.b.y = 9;
    printf("%g\n", (double)line.b.y);
    printf("%g\n", (double)((void *)&line.a == (void *)&line));
    printf("%g\n", (double)((void *)&line.a.x == (void *)&line.a));
    return 0;
}
