use crate::flatgfa::FlatGFA;
use crate::memfile;
use crate::namemap::NameMap;
use flate2::read::GzDecoder;
use memchr::memchr;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};

/// Open a GAF file reader for gzip-compressed files.
///
/// This function opens a .gaf.gz file and returns a boxed BufRead trait object
/// for streaming decompression. Plain text files should use memory mapping instead.
///
/// # Arguments
/// * `path` - The path to a .gaf.gz file
///
/// # Returns
/// A Result containing a boxed trait object implementing BufRead
fn open_gzip_reader(path: &str) -> io::Result<Box<dyn BufRead>> {
    let file = File::open(path)?;
    let decoder = GzDecoder::new(file);
    Ok(Box::new(BufReader::new(decoder)))
}

/// Process a GAF stream (implementing BufRead) and update the pangenotype matrix.
///
/// This is a generic function that works with any BufRead source (files, gzip streams, stdin, etc.)
///
/// # Arguments
/// * `reader` - A boxed BufRead trait object
/// * `matrix` - The pangenotype matrix to update
/// * `file_idx` - The index of this file in the matrix
/// * `name_map` - The name map for segment lookup
fn process_gaf_stream(
    reader: Box<dyn BufRead>,
    matrix: &mut Vec<Vec<bool>>,
    file_idx: usize,
    name_map: &crate::namemap::NameMap,
) -> io::Result<()> {
    for line in reader.lines() {
        let line = line?;

        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        process_gaf_line(&line, matrix, file_idx, name_map);
    }
    Ok(())
}

pub fn make_pangenotype_matrix(gfa: &FlatGFA, gaf_files: Vec<String>) -> Vec<Vec<bool>> {
    let num_segments = gfa.segs.len();
    let mut matrix = vec![vec![false; num_segments]; gaf_files.len()];

    let name_map = NameMap::build(&gfa);

    for (file_idx, gaf_path) in gaf_files.iter().enumerate() {
        if gaf_path.ends_with(".gz") {
            // Use BufRead stream for gzip files
            match open_gzip_reader(gaf_path) {
                Ok(reader) => {
                    if let Err(e) = process_gaf_stream(reader, &mut matrix, file_idx, &name_map) {
                        eprintln!("Error processing GAF stream {}: {}", gaf_path, e);
                    }
                }
                Err(e) => {
                    eprintln!("Error opening GAF file {}: {}", gaf_path, e);
                }
            }
        } else {
            // Use memory mapping for plain .gaf files (faster)
            let mmap = memfile::map_file(gaf_path);
            let mut start = 0;

            while let Some(pos) = memchr(b'\n', &mmap[start..]) {
                let line_end = start + pos;
                let line = &mmap[start..line_end];
                start = line_end + 1;

                if line.is_empty() || line[0] == b'#' {
                    continue;
                }

                process_gaf_line_bytes(line, &mut matrix, file_idx, &name_map);
            }
        }
    }

    matrix
}

pub fn make_pangenotype_matrix_from_stream(
    gfa: &FlatGFA,
    reader: Box<dyn BufRead>,
) -> io::Result<Vec<bool>> {
    let num_segments = gfa.segs.len();
    let mut matrix = vec![vec![false; num_segments]];
    let name_map = NameMap::build(&gfa);

    process_gaf_stream(reader, &mut matrix, 0, &name_map)?;

    Ok(matrix.remove(0))
}

fn process_gaf_line(
    line: &str,
    matrix: &mut Vec<Vec<bool>>,
    file_idx: usize,
    name_map: &crate::namemap::NameMap,
) {
    let bytes = line.as_bytes();
    let mut tab_count = 0;
    let mut idx = 0;
    while idx < bytes.len() && tab_count < 5 {
        if bytes[idx] == b'\t' {
            tab_count += 1;
        }
        idx += 1;
    }

    if tab_count < 5 || idx >= bytes.len() {
        return;
    }
    // The path data is in the 5th field (index 4, after 5 tabs)
    let mut end_idx = idx;
    while end_idx < bytes.len() && bytes[end_idx] != b'\t' {
        end_idx += 1;
    }
    let path_field = &bytes[idx..end_idx];

    parse_path_field(path_field, matrix, file_idx, name_map);
}

fn process_gaf_line_bytes(
    line: &[u8],
    matrix: &mut Vec<Vec<bool>>,
    file_idx: usize,
    name_map: &crate::namemap::NameMap,
) {
    let mut tab_count = 0;
    let mut idx = 0;
    while idx < line.len() && tab_count < 5 {
        if line[idx] == b'\t' {
            tab_count += 1;
        }
        idx += 1;
    }

    if tab_count < 5 || idx >= line.len() {
        return;
    }

    let mut end_idx = idx;
    while end_idx < line.len() && line[end_idx] != b'\t' {
        end_idx += 1;
    }
    let path_field = &line[idx..end_idx];

    parse_path_field(path_field, matrix, file_idx, name_map);
}

fn parse_path_field(
    path_field: &[u8],
    matrix: &mut Vec<Vec<bool>>,
    file_idx: usize,
    name_map: &crate::namemap::NameMap,
) {
    let mut p = 0;
    while p < path_field.len() {
        let byte = path_field[p];
        if byte == b'>' || byte == b'<' {
            p += 1;
            let mut num = 0usize;
            while p < path_field.len() && path_field[p].is_ascii_digit() {
                num = num * 10 + (path_field[p] - b'0') as usize;
                p += 1;
            }
            let seg_id = name_map.get(num);
            matrix[file_idx][u32::from(seg_id) as usize] = true;
        } else {
            p += 1;
        }
    }
}
