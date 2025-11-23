use crate::flatgfa::FlatGFA;
use crate::memfile;
use crate::namemap::NameMap;
use memchr::memchr;

pub fn make_pangenotype_matrix(gfa: &FlatGFA, gaf_files: Vec<String>) -> Vec<Vec<bool>> {
    let num_segments = gfa.segs.len();
    let mut matrix = vec![vec![false; num_segments]; gaf_files.len()];

    let name_map = NameMap::build(&gfa);

    for (file_idx, gaf_path) in gaf_files.iter().enumerate() {
        let mmap = memfile::map_file(gaf_path);
        let mut start = 0;

        while let Some(pos) = memchr(b'\n', &mmap[start..]) {
            let line_end = start + pos;
            let line = &mmap[start..line_end];
            start = line_end + 1;

            if line.is_empty() || line[0] == b'#' {
                continue;
            }

            let mut tab_count = 0;
            let mut idx = 0;
            while idx < line.len() && tab_count < 5 {
                if line[idx] == b'\t' {
                    tab_count += 1;
                }
                idx += 1;
            }

            if tab_count < 5 || idx >= line.len() {
                continue;
            }
            //The path data is in the 5th field only.
            let mut end_idx = idx;
            while end_idx < line.len() && line[end_idx] != b'\t' {
                end_idx += 1;
            }
            let path_field = &line[idx..end_idx];

            // === Parse path field like >12<34>56 ===
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
    }

    matrix
}
