use crate::flatgfa::{self, GFABuilder};
use crate::pool::{self, Index, Pool};
use argh::FromArgs;
use std::collections::HashMap;

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

/// A helper to construct a new graph that includes part of an old graph.
struct SubgraphBuilder<'a> {
    old: &'a flatgfa::FlatGFA<'a>,
    store: flatgfa::HeapStore,
    seg_map: HashMap<Index, Index>,
}

impl<'a> SubgraphBuilder<'a> {
    fn new(old: &'a flatgfa::FlatGFA) -> Self {
        Self {
            old,
            store: flatgfa::HeapStore::default(),
            seg_map: HashMap::new(),
        }
    }

    /// Add a segment from the source graph to this subgraph.
    fn include_seg(&mut self, seg_id: Index) {
        let seg = &self.old.segs[seg_id as usize];
        let new_seg_id = self.store.add_seg(
            seg.name,
            self.old.get_seq(seg),
            self.old.get_optional_data(seg),
        );
        self.seg_map.insert(seg_id, new_seg_id);
    }

    /// Translate a handle from the source graph to this subgraph.
    fn tr_handle(&self, old_handle: flatgfa::Handle) -> flatgfa::Handle {
        flatgfa::Handle::new(self.seg_map[&old_handle.segment()], old_handle.orient())
    }
}

pub fn extract(gfa: &flatgfa::FlatGFA, args: Extract) {
    // Find the segment.
    // TODO: Nicer error handling.
    let origin_seg = gfa.find_seg(args.seg_name).expect("segment not found");

    let mut subgraph = SubgraphBuilder::new(gfa);
    subgraph.include_seg(origin_seg);

    // Find the set of all segments that are 1 link away, and insert them into a new
    // subgraph.
    assert_eq!(args.link_distance, 1, "only `-c 1` is implemented so far");
    for link in gfa.links.iter() {
        if let Some(other_seg) = link.incident_seg(origin_seg) {
            if !subgraph.seg_map.contains_key(&other_seg) {
                subgraph.include_seg(other_seg);
            }
        }
    }

    // Create a new graph with only segments, paths, and indices that "touch"
    // the neighborhood.

    for link in gfa.links.iter() {
        if subgraph.seg_map.contains_key(&link.from.segment())
            && subgraph.seg_map.contains_key(&link.to.segment())
        {
            let from = subgraph.tr_handle(link.from);
            let to = subgraph.tr_handle(link.to);
            let overlap = gfa.get_alignment(&link.overlap);
            subgraph.store.add_link(from, to, overlap.ops.into());
        }
    }

    struct SubpathStart {
        step: Index, // The id of the first step in the subpath.
        pos: usize,  // The bp position at the start of the subpath.
    }

    // TODO: We should offer Pool-like iteration abstractions for all the sets... this
    // would make it less error-prone to get the id and path together, for example.
    for path in gfa.paths.iter() {
        let mut cur_subpath_start: Option<SubpathStart> = None;
        let mut path_pos = 0;

        for step in gfa.get_steps(path) {
            let in_neighb = subgraph.seg_map.contains_key(&step.segment());
            match (&cur_subpath_start, in_neighb) {
                (Some(_), true) => {} // continue
                (Some(start), false) => {
                    // End the current subpath.
                    let steps = pool::Span {
                        start: start.step,
                        end: subgraph.store.steps.next_id(),
                    };
                    let name = format!("{}:{}-{}", gfa.get_path_name(&path), start.pos, path_pos);
                    subgraph
                        .store
                        .add_path(name.as_bytes(), steps, std::iter::empty());
                    cur_subpath_start = None;
                }
                (None, true) => {
                    // Starting a new subpath.
                    cur_subpath_start = Some(SubpathStart {
                        step: subgraph.store.steps.next_id(),
                        pos: path_pos,
                    });
                }
                (None, false) => {} // not in the ballpark
            }

            // Add the (translated) step to the new graph.
            if in_neighb {
                let handle = flatgfa::Handle::new(subgraph.seg_map[&step.segment()], step.orient());
                subgraph.store.add_step(handle);
            }

            // Track the current bp position in the path.
            path_pos += gfa.get_handle_seg(*step).len();
        }

        // Did we reach the end of the path while still in the neighborhood?
        if let Some(start) = cur_subpath_start {
            // TODO Reduce duplication!
            let steps = pool::Span {
                start: start.step,
                end: subgraph.store.steps.next_id(),
            };
            let name = format!("{}:{}-{}", gfa.get_path_name(&path), start.pos, path_pos);
            subgraph
                .store
                .add_path(name.as_bytes(), steps, std::iter::empty());
        }
    }

    // TODO: It would be great to be able to emit FlatGFA files instead too.
    crate::print::print(&subgraph.store.view());
}
