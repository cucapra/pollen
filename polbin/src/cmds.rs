use crate::flatgfa::{self, GFABuilder};
use crate::pool::Index;
use argh::FromArgs;
use std::collections::{HashMap, HashSet};

/// print the FlatGFA table of contents
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "toc")]
pub struct Toc {}

pub fn toc(gfa: &flatgfa::FlatGFA) {
    eprintln!("header: {}", gfa.header.len());
    eprintln!("segs: {}", gfa.segs.len());
    eprintln!("paths: {}", gfa.paths.len());
    eprintln!("links: {}", gfa.links.len());
    eprintln!("steps: {}", gfa.steps.len());
    eprintln!("seq_data: {}", gfa.seq_data.len());
    eprintln!("overlaps: {}", gfa.overlaps.len());
    eprintln!("alignment: {}", gfa.alignment.len());
    eprintln!("name_data: {}", gfa.name_data.len());
    eprintln!("optional_data: {}", gfa.optional_data.len());
    eprintln!("line_order: {}", gfa.line_order.len());
}

/// list the paths
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "paths")]
pub struct Paths {}

pub fn paths(gfa: &flatgfa::FlatGFA) {
    for path in gfa.paths.iter() {
        println!("{}", gfa.get_path_name(path));
    }
}

/// calculate graph statistics
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "stats")]
pub struct Stats {
    /// show basic metrics
    #[argh(switch, short = 'S')]
    summarize: bool,

    /// number of segments with at least one self-loop link
    #[argh(switch, short = 'L')]
    self_loops: bool,
}

pub fn stats(gfa: &flatgfa::FlatGFA, args: Stats) {
    if args.summarize {
        println!("#length\tnodes\tedges\tpaths\tsteps");
        println!(
            "{}\t{}\t{}\t{}\t{}",
            gfa.seq_data.len(),
            gfa.segs.len(),
            gfa.links.len(),
            gfa.paths.len(),
            gfa.steps.len()
        );
    } else if args.self_loops {
        let mut counts: HashMap<Index, usize> = HashMap::new();
        let mut total: usize = 0;
        for link in gfa.links.iter() {
            if link.from.segment() == link.to.segment() {
                let count = counts.entry(link.from.segment()).or_insert(0);
                *count += 1;
                total += 1;
            }
        }
        println!("#type\tnum");
        println!("total\t{}", total);
        println!("unique\t{}", counts.len());
    }
}

/// create a subset graph
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "extract")]
pub struct Extract {
    /// segment to extract around
    #[argh(option, short = 'n')]
    seg_name: usize,

    /// number of edges "away" from the node to include
    #[argh(option, short = 'c')]
    link_distance: usize,
}

pub fn extract(gfa: &flatgfa::FlatGFA, args: Extract) {
    // Find the segment.
    // TODO: Maybe we should maintain an index? Or at least provide a helper for this?
    let origin_seg = gfa.segs.iter().position(|seg| seg.name == args.seg_name);
    let origin_seg = origin_seg.expect("segment not found") as Index; // TODO Nicer error reporting.

    assert_eq!(args.link_distance, 1, "only `-c 1` is implemented so far");

    // Find the set of all segments that are 1 link away.
    let mut neighborhood = HashSet::new();
    neighborhood.insert(origin_seg);
    for link in gfa.links.iter() {
        if let Some(other_seg) = link.incident_seg(origin_seg) {
            neighborhood.insert(other_seg);
        }
    }

    // Create a new graph with only segments, paths, and indices that "touch"
    // the neighborhood.
    let mut store = flatgfa::HeapStore::default();

    let mut seg_id_map = HashMap::new();
    for seg_id in neighborhood.iter() {
        let seg = &gfa.segs[*seg_id as usize];
        let new_seg_id = store.add_seg(seg.name, gfa.get_seq(seg), gfa.get_optional_data(seg));
        seg_id_map.insert(*seg_id, new_seg_id);
    }

    for link in gfa.links.iter() {
        if neighborhood.contains(&link.from.segment()) && neighborhood.contains(&link.to.segment())
        {
            // TODO Lots of repetition to be reduced here. It would be great if we could make
            // the ID translation kinda transparent, somehow...
            let from = flatgfa::Handle::new(seg_id_map[&link.from.segment()], link.from.orient());
            let to = flatgfa::Handle::new(seg_id_map[&link.to.segment()], link.to.orient());
            let overlap = gfa.get_alignment(&link.overlap);
            store.add_link(from, to, overlap.ops.into());
        }
    }

    let mut neighb_paths = HashSet::new();
    // TODO: We should offer Pool-like iteration abstractions for all the sets... this
    // would make it less error-prone to get the id and path together, for example.
    for (id, path) in gfa.paths.iter().enumerate() {
        for step in gfa.get_steps(path) {
            if neighborhood.contains(&step.segment()) {
                neighb_paths.insert(id as Index);
                break;
            }
        }
    }
    for path_id in neighb_paths {
        let path = gfa.paths[path_id as usize];
        let steps = store.add_steps(gfa.get_steps(&path).iter().filter_map(|step| {
            if neighborhood.contains(&step.segment()) {
                let seg = seg_id_map[&step.segment()];
                Some(flatgfa::Handle::new(seg, step.orient()))
            } else {
                // TODO We could just stop iterating here? Since the path is guaranteed
                // not to cross a second time...
                None
            }
        }));
        // TODO: Copying all of this is incredibly inefficient; we could do more direct
        // copying of the ranges of overlaps & alignments.
        // TODO: This is also incorrect! We should only copy the alignments from the steps
        // we actually kept.
        let overlaps = gfa
            .get_overlaps(&path)
            .iter()
            .map(|s| gfa.get_alignment(s).ops.to_owned());
        store.add_path(&gfa.get_path_name(&path), steps, overlaps);
    }

    crate::print::print(&store.view());
}
