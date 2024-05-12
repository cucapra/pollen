use crate::flatgfa::{self, Handle, Segment};
use crate::pool::{self, Id, Store};
use argh::FromArgs;
use bstr::BStr;
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
    for path in gfa.paths.all().iter() {
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
        let mut counts: HashMap<Id<Segment>, usize> = HashMap::new();
        let mut total: usize = 0;
        for link in gfa.links.all().iter() {
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

/// find a nucleotide position within a path
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "position")]
pub struct Position {
    /// path_name,offset,orientation
    #[argh(option, short = 'p')]
    path_pos: String,
}

pub fn position(gfa: &flatgfa::FlatGFA, args: Position) -> Result<(), &'static str> {
    // Parse the position triple, which looks like `path,42,+`.
    let (path_name, offset, orientation) = {
        let parts: Vec<_> = args.path_pos.split(',').collect();
        if parts.len() != 3 {
            return Err("position must be path_name,offset,orientation");
        }
        let off: usize = parts[1].parse().or(Err("offset must be a number"))?;
        let ori: flatgfa::Orientation = parts[2].parse().or(Err("orientation must be + or -"))?;
        (parts[0], off, ori)
    };

    let path_id = gfa.find_path(path_name.into()).ok_or("path not found")?;
    let path = &gfa.paths[path_id];
    assert_eq!(
        orientation,
        flatgfa::Orientation::Forward,
        "only + is implemented so far"
    );

    // Traverse the path until we reach the position.
    let mut cur_pos = 0;
    let mut found = None;
    for step in &gfa.steps[path.steps] {
        let seg = gfa.get_handle_seg(*step);
        let end_pos = cur_pos + seg.len();
        if offset < end_pos {
            // Found it!
            found = Some((*step, offset - cur_pos));
            break;
        }
        cur_pos = end_pos;
    }

    // Print the match.
    if let Some((handle, seg_off)) = found {
        let seg = gfa.get_handle_seg(handle);
        let seg_name = seg.name;
        println!("#source.path.pos\ttarget.graph.pos");
        println!(
            "{},{},{}\t{},{},{}",
            path_name,
            offset,
            orientation,
            seg_name,
            seg_off,
            handle.orient()
        );
    }

    Ok(())
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

pub fn extract(
    gfa: &flatgfa::FlatGFA,
    args: Extract,
) -> Result<flatgfa::HeapGFAStore, &'static str> {
    let origin_seg = gfa.find_seg(args.seg_name).ok_or("segment not found")?;

    let mut subgraph = SubgraphBuilder::new(gfa);
    subgraph.extract(origin_seg, args.link_distance);
    Ok(subgraph.store)
}

/// A helper to construct a new graph that includes part of an old graph.
struct SubgraphBuilder<'a> {
    old: &'a flatgfa::FlatGFA<'a>,
    store: flatgfa::HeapGFAStore,
    seg_map: HashMap<Id<Segment>, Id<Segment>>,
}

struct SubpathStart {
    step: Id<Handle>, // The id of the first step in the subpath.
    pos: usize,       // The bp position at the start of the subpath.
}

impl<'a> SubgraphBuilder<'a> {
    fn new(old: &'a flatgfa::FlatGFA) -> Self {
        Self {
            old,
            store: flatgfa::HeapGFAStore::default(),
            seg_map: HashMap::new(),
        }
    }

    /// Add a segment from the source graph to this subgraph.
    fn include_seg(&mut self, seg_id: Id<Segment>) {
        let seg = &self.old.segs[seg_id];
        let new_seg_id = self.store.add_seg(
            seg.name,
            self.old.get_seq(seg),
            self.old.get_optional_data(seg),
        );
        self.seg_map.insert(seg_id, new_seg_id);
    }

    /// Add a link from the source graph to the subgraph.
    fn include_link(&mut self, link: &flatgfa::Link) {
        let from = self.tr_handle(link.from);
        let to = self.tr_handle(link.to);
        let overlap = self.old.get_alignment(link.overlap);
        self.store.add_link(from, to, overlap.ops.into());
    }

    /// Add a single subpath from the given path to the subgraph.
    fn include_subpath(&mut self, path: &flatgfa::Path, start: &SubpathStart, end_pos: usize) {
        let steps = pool::Span::new(start.step, self.store.steps.next_id());
        let name = format!("{}:{}-{}", self.old.get_path_name(path), start.pos, end_pos);
        self.store
            .add_path(name.as_bytes(), steps, std::iter::empty());
    }

    /// Identify all the subpaths in a path from the original graph that cross through
    /// segments in this subgraph and add them.
    fn find_subpaths(&mut self, path: &flatgfa::Path) {
        let mut cur_subpath_start: Option<SubpathStart> = None;
        let mut path_pos = 0;

        for step in &self.old.steps[path.steps] {
            let in_neighb = self.seg_map.contains_key(&step.segment());

            if let (Some(start), false) = (&cur_subpath_start, in_neighb) {
                // End the current subpath.
                self.include_subpath(path, start, path_pos);
                cur_subpath_start = None;
            } else if let (None, true) = (&cur_subpath_start, in_neighb) {
                // Start a new subpath.
                cur_subpath_start = Some(SubpathStart {
                    step: self.store.steps.next_id(),
                    pos: path_pos,
                });
            }

            // Add the (translated) step to the new graph.
            if in_neighb {
                self.store.add_step(self.tr_handle(*step));
            }

            // Track the current bp position in the path.
            path_pos += self.old.get_handle_seg(*step).len();
        }

        // Did we reach the end of the path while still in the neighborhood?
        if let Some(start) = cur_subpath_start {
            self.include_subpath(path, &start, path_pos);
        }
    }

    /// Translate a handle from the source graph to this subgraph.
    fn tr_handle(&self, old_handle: flatgfa::Handle) -> flatgfa::Handle {
        flatgfa::Handle::new(self.seg_map[&old_handle.segment()], old_handle.orient())
    }

    /// Check whether a segment from the old graph is in the subgraph.
    fn contains(&self, old_seg_id: Id<Segment>) -> bool {
        self.seg_map.contains_key(&old_seg_id)
    }

    /// Extract a subgraph consisting of a neighborhood of segments up to `dist` links away
    /// from the given segment in the original graph.
    ///
    /// Include any links between the segments in the neighborhood and subpaths crossing
    /// through the neighborhood.
    fn extract(&mut self, origin: Id<Segment>, dist: usize) {
        self.include_seg(origin);

        // Find the set of all segments that are 1 link away.
        assert_eq!(dist, 1, "only `-c 1` is implemented so far");
        for link in self.old.links.all().iter() {
            if let Some(other_seg) = link.incident_seg(origin) {
                if !self.seg_map.contains_key(&other_seg) {
                    self.include_seg(other_seg);
                }
            }
        }

        // Include all links within the subgraph.
        for link in self.old.links.all().iter() {
            if self.contains(link.from.segment()) && self.contains(link.to.segment()) {
                self.include_link(link);
            }
        }

        // Find subpaths within the subgraph.
        for path in self.old.paths.all().iter() {
            self.find_subpaths(path);
        }
    }
}

/// compute node depth, the number of times paths cross a node
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "depth")]
pub struct Depth {}

pub fn depth(gfa: &flatgfa::FlatGFA) {
    // Initialize node depth
    let mut depths = vec![0; gfa.segs.len()];
    // Initialize uniq_paths
    let mut uniq_paths = Vec::<HashSet<&BStr>>::new();
    uniq_paths.resize(gfa.segs.len(), HashSet::new());
    // do not assume that each handle in `gfa.steps()` is unique
    for path in gfa.paths.all() {
        let path_name = gfa.get_path_name(path);
        for step in &gfa.steps[path.steps] {
            let seg_id = step.segment().index();
            // Increment depths
            depths[seg_id] += 1;
            // Update uniq_paths
            uniq_paths[seg_id].insert(path_name);
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
