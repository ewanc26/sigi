#include <stdio.h>
#include <stdint.h>

struct Rec {
    int32_t key;
    double weight;
};

static struct Rec table[5];

int main(void) {
    table[0].key = 42;
    table[0].weight = 1.5;
    table[1].key = 7;
    table[1].weight = 9.25;
    table[2].key = 19;
    table[2].weight = 3;
    table[3].key = 3;
    table[3].weight = 0.5;
    table[4].key = 27;
    table[4].weight = 6.75;

    for (int i = 0; i < 4; i++) {
        int m = i;
        for (int j = i + 1; j < 5; j++) {
            if (table[j].key < table[m].key) {
                m = j;
            }
        }
        int32_t kmin = table[m].key;
        int32_t kcur = table[i].key;
        table[i].key = kmin;
        table[m].key = kcur;
        double wmin = table[m].weight;
        double wcur = table[i].weight;
        table[i].weight = wmin;
        table[m].weight = wcur;
    }

    printf("key weight\n");
    for (int i = 0; i < 5; i++) {
        printf("%d\n", table[i].key);
        printf("%g\n", table[i].weight);
    }
    return 0;
}
