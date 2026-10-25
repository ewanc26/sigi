#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>

struct Step {
    int32_t op;
    double a;
    double b;
};

typedef double (*binop)(double, double);

static double add(double a, double b) { return a + b; }
static double sub(double a, double b) { return a - b; }
static double mul(double a, double b) { return a * b; }

static double divide(double a, double b) {
    if (b == 0) {
        exit(4);
    }
    return a / b;
}

static binop table[4] = { add, sub, mul, divide };

static struct Step prog[4] = {
    { 0, 12, 30 },
    { 2, 6, 7 },
    { 1, 100, 58 },
    { 3, 84, 0 },
};

int main(void) {
    for (int pc = 0; pc < 4; pc++) {
        double r = table[prog[pc].op](prog[pc].a, prog[pc].b);
        printf("%g\n", r);
    }
    return 0;
}
