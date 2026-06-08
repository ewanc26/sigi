#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <time.h>
#include <unistd.h>

#define STACK_SIZE 1000
#define MAX_ARRAYS 10

static double stack[STACK_SIZE];
static int sp = 0;
static double vars[100];
static double *arrays[MAX_ARRAYS];
static int array_sizes[MAX_ARRAYS];

static void arr_init(int id, int size) {
    if (id < 0 || id >= MAX_ARRAYS) { fprintf(stderr, "Array ID out of range\n"); exit(1); }
    arrays[id] = (double *)calloc(size, sizeof(double));
    array_sizes[id] = size;
}

static void arr_free(int id) {
    if (id < 0 || id >= MAX_ARRAYS) { fprintf(stderr, "Array ID out of range\n"); exit(1); }
    if (arrays[id]) { free(arrays[id]); arrays[id] = NULL; array_sizes[id] = 0; }
}

static void push(double x) {
    if (sp >= STACK_SIZE) { fprintf(stderr, "Stack overflow\n"); exit(1); }
    stack[sp++] = x;
}

static double pop(void) {
    if (sp <= 0) { fprintf(stderr, "Stack underflow\n"); exit(1); }
    return stack[--sp];
}
