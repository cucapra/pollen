use std::fs;
use std::fs::File;
use std::io::BufReader;

use crate::ops::depth::{self, *};
use crate::{flatgfa, parse::*};
use crate::{memfile::*, FlatGFA};

struct SegmentDepth {
    depth: f64,
    range: (usize, usize),
}

fn path_length(gfa: &FlatGFA) -> usize {
    let path = gfa.paths.all()[0];
    let mut total_len = 0;
    for step in &gfa.steps[path.steps] {
        let seg_id = step.segment().index();
        total_len += gfa.segs.all()[seg_id].seq.len();
    }
    return total_len;
}

fn compute_windows(path_len: usize, window_size: usize) -> Vec<(usize, usize)> {
    let mut start = 0;
    let mut windows: Vec<(usize, usize)> = Vec::new();
    while start < path_len {
        let end = (start + window_size).min(path_len);
        windows.push((start, end));
        start = end;
    }
    windows
}

fn compute_depths<'a>(gfa: &'a FlatGFA<'a>, depth: &'a Vec<usize>) -> Vec<SegmentDepth> {
    let mut depths: Vec<SegmentDepth> = Vec::new();
    let mut pos = 0;
    let steps_span = gfa.paths.all()[0].steps;
    let steps = &gfa.steps.all()[steps_span.start.index()..steps_span.end.index()];
    for step in steps {
        let segment = gfa.segs[step.segment()];
        let old_pos = pos;
        pos += segment.len();
        let total = depth[step.segment().index()] * segment.len();
        let seg_depth = SegmentDepth {
            depth: total as f64,
            range: (old_pos, pos),
        };

        depths.push(seg_depth);
    }
    depths
}

fn assign_depths(seg_depth: &Vec<SegmentDepth>, windows: &Vec<(usize, usize)>) -> Vec<f64> {
    let mut depths: Vec<f64> = vec![0.0; windows.len()];
    for seg in seg_depth {
        for i in 0..windows.len() {
            let window = windows[i];
            let start = window.0.max(seg.range.0) as i32;
            let end = window.1.min(seg.range.1) as i32;
            let overlap = end - start;
            if overlap > 0 {
                let overlap_amt: f64 = overlap as f64 / (seg.range.1 - seg.range.0) as f64;
                depths[i] += (seg.depth * overlap_amt) / ((window.1 - window.0) as f64);
            }
        }
    }
    depths
}

fn format_float(x: f64) -> String {
    let s = format!("{:.6}", x);
    let s = s.trim_end_matches('0');
    let s = s.trim_end_matches('.');
    s.to_string()
}

pub fn create_bed(
    flatgfa: &flatgfa::FlatGFA,
    bed_name: &str,
    chrom_name: &str,
    window_size: usize,
    window_vec: Vec<(usize, usize)>,
) {
    let path_len = path_length(&flatgfa);
    let num_windows = path_len.div_ceil(window_size);
    let max_line = 40;
    let mut bed_file = map_new_file(bed_name, (num_windows * max_line) as u64);
    let mut offset = 0;
    let depth = depth(&flatgfa).0;
    let mut windows = window_vec;
    if windows == Vec::new() {
        windows = compute_windows(path_len, window_size);
    }
    let seg_depths = compute_depths(&flatgfa, &depth);
    let depths_final = assign_depths(&seg_depths, &windows);
    for i in 0..windows.len() {
        let start = windows[i].0;
        let end = windows[i].1;
        let depth_str: String = format_float(depths_final[i]);
        let line = format!("{chrom_name}\t{start}\t{end}\t{depth_str}\n");
        let bytes = line.as_bytes();
        bed_file[offset..offset + bytes.len()].copy_from_slice(bytes);
        offset += bytes.len();
    }
    bed_file.flush().unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    // #[test]
    // fn test_windows() {
    //     let gfa_file_name = "k_copy.gfa";
    //     let bed_file_name = "windows_test";
    //     let chrom_name = "x";
    //     let window_size = 5;
    //     create_bed(
    //         gfa_file_name,
    //         bed_file_name,
    //         chrom_name,
    //         window_size,
    //         Vec::new(),
    //     );
    //     let content = fs::read_to_string(bed_file_name).unwrap();
    //     println!("{}", content);
    //     assert_eq!(0, 0);
    // }
}
