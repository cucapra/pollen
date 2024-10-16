use crate::fgfa_ds::flatgfa::FlatGFA;
use argh::FromArgs;
use std::collections::HashSet;

/// compute node depth, the number of times paths cross a node
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "depth")]
pub struct Depth {}

pub fn depth(gfa: &FlatGFA) {
    // Initialize node depth
    let mut depths = vec![0; gfa.segs.len()];
    // Initialize uniq_paths
    let mut uniq_paths = Vec::<HashSet<usize>>::new();
    uniq_paths.resize(gfa.segs.len(), HashSet::new());
    // do not assume that each handle in `gfa.steps()` is unique
    for (idx, path) in gfa.paths.all().iter().enumerate() {
        for step in &gfa.steps[path.steps] {
            let seg_id = step.segment().index();
            // Increment depths
            depths[seg_id] += 1;
            // Update uniq_paths
            uniq_paths[seg_id].insert(idx);
        }
    }
    // print out depth and depth.uniq
    println!("#node.id\tdepth\tdepth.uniq");
    for (id, seg) in gfa.segs.items() {
        let name: u32 = seg.name as u32;
        println!(
            "{}\t{}\t{}",
            name,
            depths[id.index()],
            uniq_paths[id.index()].len()
        );
    }
}