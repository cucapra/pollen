#include <stdio.h>
#include "../include/flatgfa.h"

int main(int argc, const char** argv) {
    if (argc <= 1) {
        fprintf(stderr, "usage: %s file.gfa\n", argv[0]);
        exit(1);
    }
    const char* filename = argv[1];
    flatgfa_parse("../../tests/k.gfa");
}
