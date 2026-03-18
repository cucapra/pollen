#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct FlatGFAHandle FlatGFAHandle;

/**
 * A single step in a path: a segment ID and an orientation.
 */
typedef struct CStep {
  uint32_t segment_id;
  bool is_forward;
} CStep;

void hello_world(void);

/**
 * Parse a GFA file and return an opaque handle.
 * Caller must free with flatgfa_free().
 */
struct FlatGFAHandle *flatgfa_parse(const char *filename);

/**
 * Free a FlatGFA handle.
 */
void flatgfa_free(struct FlatGFAHandle *gfa);

uintptr_t flatgfa_path_count(const struct FlatGFAHandle *gfa);

/**
 * Get the name of a path by index. Returns a pointer to the name bytes (not
 * null-terminated) and sets `*len` to the byte length. The pointer is valid
 * as long as the FlatGFAHandle is alive. Returns null if index is out of bounds.
 */
const uint8_t *flatgfa_get_path_name(const struct FlatGFAHandle *gfa,
                                     uintptr_t path_index,
                                     uintptr_t *len);

/**
 * Get the number of steps in a path. Returns usize::MAX if index is out of bounds.
 */
uintptr_t flatgfa_get_path_step_count(const struct FlatGFAHandle *gfa, uintptr_t path_index);

/**
 * Get a single step from a path by path index and step index. Returns true on
 * success and writes into `*out`. Returns false if either index is out of bounds.
 */
bool flatgfa_get_step(const struct FlatGFAHandle *gfa,
                      uintptr_t path_index,
                      uintptr_t step_index,
                      struct CStep *out);

/**
 * Get number of DNA sequences in GFA file
 */
uintptr_t flatgfa_get_segment_count(const struct FlatGFAHandle *gfa);

/**
 * Get the DNA sequence for a segment. Returns a pointer to the raw bytes (not
 * null-terminated) and sets `*len`. The pointer is valid as long as the
 * FlatGFAHandle is alive. Returns null if segment_id is out of bounds.
 * Note: always returns the forward-strand sequence regardless of orientation.
 */
const uint8_t *flatgfa_get_segment_seq(const struct FlatGFAHandle *gfa,
                                       uint32_t segment_id,
                                       uintptr_t *len);
