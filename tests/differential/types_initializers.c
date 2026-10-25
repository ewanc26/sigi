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

static struct Pt origin = { 2.5, 7 };
static struct Seg line = { { 1, 2 }, { 3, 0 } };
static struct Pt zero = { };
static struct Pt typed = { 2.5, 9 };

int main(void) {
    printf("%g\n", origin.x);
    printf("%g\n", (double)origin.y);
    printf("%g\n", line.a.x);
    printf("%g\n", (double)line.a.y);
    printf("%g\n", line.b.x);
    printf("%g\n", (double)line.b.y);
    printf("%g\n", zero.x);
    printf("%g\n", (double)zero.y);
    printf("%g\n", (double)typed.y);
    return 0;
}
