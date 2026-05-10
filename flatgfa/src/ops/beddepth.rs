use crate::flatgfa;
use crate::ops::depth::{format_float, seg_depth};
use crate::ops::windows::compute_windows;
use crate::FlatGFA;

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
    total_len
}

fn compute_depths<'a>(gfa: &'a FlatGFA<'a>, depth: &'a [usize]) -> Vec<SegmentDepth> {
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

#[allow(clippy::mut_range_bound)]
fn assign_depths(seg_depth: &Vec<SegmentDepth>, windows: &[(usize, usize)]) -> Vec<f64> {
    let mut depths: Vec<f64> = vec![0.0; windows.len()];
    let mut cur_window_idx = 0;
    let mut overlap_flag = false;
    for seg in seg_depth {
        for i in cur_window_idx..windows.len() {
            let window = windows[i];
            let start = window.0.max(seg.range.0) as i32;
            let end = window.1.min(seg.range.1) as i32;
            let overlap = end - start;
            if overlap > 0 {
                let overlap_amt: f64 = overlap as f64 / (seg.range.1 - seg.range.0) as f64;
                depths[i] += (seg.depth * overlap_amt) / ((window.1 - window.0) as f64);
                cur_window_idx = i;
                overlap_flag = true;
            } else if overlap_flag {
                overlap_flag = false;
                break;
            }
        }
    }
    depths
}

pub fn create_bed(flatgfa: &flatgfa::FlatGFA, chrom_name: &str, window_size: usize) {
    let path_len = path_length(flatgfa);
    let depth = seg_depth(flatgfa).0;
    let windows = compute_windows(path_len, window_size);
    let seg_depths = compute_depths(flatgfa, &depth);
    let depths_final = assign_depths(&seg_depths, &windows);
    for i in 0..windows.len() {
        let start = windows[i].0;
        let end = windows[i].1;
        let depth_str: String = format_float(depths_final[i]);
        println!("{chrom_name}\t{start}\t{end}\t{depth_str}");
    }
}
