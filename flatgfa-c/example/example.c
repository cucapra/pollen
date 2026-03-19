#include <stdint.h>
#include <stdio.h>
#include <stdint.h>
#include <assert.h>
#include "../include/flatgfa.h"

int main(int argc, const char** argv) {
    if (argc <= 1) {
        fprintf(stderr, "usage: %s file.gfa\n", argv[0]);
        exit(1);
    }

    // Parse a GFA text file.
    const char* filename = argv[1];
    FlatGFARef* gfa = flatgfa_parse(filename);

    // Traverse all the paths.
    uint32_t path_count = flatgfa_path_count(gfa);
    for (uint32_t i = 0; i < path_count; ++i) {
        // Print the path name.
        FlatGFAString name = flatgfa_get_path_name(gfa, i);
        printf("%.*s:\n", name.len, name.data);

        // Traverse the steps in the path.
        uint32_t step_count = flatgfa_get_path_step_count(gfa, i);
        for (uint32_t j = 0; j < path_count; ++j) {
            FlatGFAHandle step;
            assert(flatgfa_get_step(gfa, i, j, &step));

            uintptr_t seq_len;
            FlatGFAString seq = flatgfa_get_seq(gfa, step.segment_id);

            // Show the direction and sequence data.
            printf("  %c %.*s\n", step.is_forward ? '+' : '-', seq.len, seq.data);
        }
    }

    // Clean up.
    flatgfa_free(gfa);

    return 0;
}
