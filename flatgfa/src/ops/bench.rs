use crate::memfile;
use rayon::iter::ParallelIterator;

// Count the lines in a file, like `wc -l`.
pub fn line_count(filename: &str, parallel: bool) -> usize {
    let buf = memfile::map_file(&filename);
    let split = memfile::MemchrSplit::new(b'\n', &buf);
    if parallel {
        ParallelIterator::count(split)
    } else {
        Iterator::count(split)
    }
}
