use crate::flatgfa::{self, Handle, Link, Orientation, Path, Segment};
use crate::memfile;
use crate::pool::{self, Id, Span, Store};
use crate::{GFAStore, HeapFamily};
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

/// benchmarks
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "bench")]
pub struct Bench {
    /// count lines in a text file
    #[argh(option)]
    wcl: Option<String>,
}

pub fn bench(args: Bench) {
    // TODO: We don't need a GFA for (some of) these? So avoid opening it.
    match args.wcl {
        Some(filename) => {
            let buf = memfile::map_file(&filename);
            let count = memfile::MemchrSplit::new(b'\n', &buf).count();
            println!("{}", count);
        }
        None => {}
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

    /// maximum number of basepairs allowed between subpaths s.t. the subpaths are merged together
    #[argh(
        option,
        short = 'd',
        long = "max-distance-subpaths",
        default = "300000"
    )]
    max_distance_subpaths: usize, // TODO: possibly make this bigger

    /// maximum number of iterations before we stop merging subpaths
    #[argh(option, short = 'e', long = "max-merging-iterations", default = "6")]
    num_iterations: usize, // TODO: probably make this smaller
}

pub fn extract(
    gfa: &flatgfa::FlatGFA,
    args: Extract,
) -> Result<flatgfa::HeapGFAStore, &'static str> {
    let origin_seg = gfa.find_seg(args.seg_name).ok_or("segment not found")?;

    let mut subgraph = SubgraphBuilder::new(gfa);
    subgraph.add_header();
    subgraph.extract(
        origin_seg,
        args.link_distance,
        args.max_distance_subpaths,
        args.num_iterations,
    );
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

    /// Include the old graph's header
    fn add_header(&mut self) {
        // pub fn add_header(&mut self, version: &[u8]) {
        //     assert!(self.header.as_ref().is_empty());
        //     self.header.add_slice(version);
        // }
        assert!(self.store.header.as_ref().is_empty());
        self.store.header.add_slice(self.old.header.all());
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
        let steps = pool::Span::new(start.step, self.store.steps.next_id()); // why the next id?
        let name = format!("{}:{}-{}", self.old.get_path_name(path), start.pos, end_pos);
        self.store
            .add_path(name.as_bytes(), steps, std::iter::empty());
    }

    /// Identify all the subpaths in a path from the original graph that cross through
    /// segments in this subgraph and merge them if possible.
    fn merge_subpaths(&mut self, path: &flatgfa::Path, max_distance_subpaths: usize) {
        // these are subpaths which *aren't* already included in the new graph
        let mut cur_subpath_start: Option<usize> = Some(0);
        let mut subpath_length = 0;
        let mut ignore_path = true;

        for (idx, step) in self.old.steps[path.steps].iter().enumerate() {
            let in_neighb = self.seg_map.contains_key(&step.segment());

            if let (Some(start), true) = (&cur_subpath_start, in_neighb) {
                // We just entered the subgraph. End the current subpath.
                if !ignore_path && subpath_length <= max_distance_subpaths {
                    // TODO: type safety
                    let subpath_span = Span::new(
                        path.steps.start + *start as u32,
                        path.steps.start + idx as u32,
                    );
                    for step in &self.old.steps[subpath_span] {
                        if !self.seg_map.contains_key(&step.segment()) {
                            self.include_seg(step.segment());
                        }
                    }
                }
                cur_subpath_start = None;
                ignore_path = false;
            } else if let (None, false) = (&cur_subpath_start, in_neighb) {
                // We've exited the current subgraph, start a new subpath
                cur_subpath_start = Some(idx);
            }

            // Track the current bp position in the path.
            subpath_length += self.old.get_handle_seg(*step).len();
        }
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
        // TODO: is this just generating the handle or should we add it to the new graph?
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
    fn extract(
        &mut self,
        origin: Id<Segment>,
        dist: usize,
        max_distance_subpaths: usize,
        num_iterations: usize,
    ) {
        self.include_seg(origin);

        // Find the set of all segments that are c links away.
        let mut frontier: Vec<Id<Segment>> = Vec::new();
        let mut next_frontier: Vec<Id<Segment>> = Vec::new();
        frontier.push(origin);
        for _ in 0..dist {
            while let Some(seg_id) = frontier.pop() {
                for link in self.old.links.all().iter() {
                    if let Some(other_seg) = link.incident_seg(seg_id) {
                        // Add other_seg to the frontier set if it is not already in the frontier set or the seg_map
                        if !self.seg_map.contains_key(&other_seg) {
                            self.include_seg(other_seg);
                            next_frontier.push(other_seg);
                        }
                    }
                }
            }
            (frontier, next_frontier) = (next_frontier, frontier);
        }

        // Merge subpaths within max_distance_subpaths bp of each other, num_iterations times
        for _ in 0..num_iterations {
            for path in self.old.paths.all().iter() {
                self.merge_subpaths(path, max_distance_subpaths);
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

/// chop the segments in a graph into sizes of N or smaller
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "chop")]
pub struct Chop {
    /// maximimum segment size.
    // Use c in keeping with odgi convention
    #[argh(option, short = 'c')]
    c: usize,

    /// compute new links
    #[argh(switch, short = 'l')]
    l: bool,
}

/// Chop a graph into segments of size no larger than c
/// By default, compact node ids
/// CIGAR strings, links, and optional Segment data are invalidated by chop
/// Generates a new graph, rather than modifying the old one in place
pub fn chop<'a>(
    gfa: &'a flatgfa::FlatGFA<'a>,
    args: Chop,
) -> Result<flatgfa::HeapGFAStore, &'static str> {
    let mut flat = flatgfa::HeapGFAStore::default();

    // when segment S is chopped into segments S1 through S2 (exclusive),
    // seg_map[S.name] = Span(Id(S1.name), Id(S2.name)). If S is not chopped: S=S1, S2.name = S1.name+1
    let mut seg_map: Vec<Span<Segment>> = Vec::new();
    // The smallest id (>0) which does not already belong to a segment in `flat`
    let mut max_node_id = 1;

    fn link_forward(flat: &mut GFAStore<'static, HeapFamily>, span: &Span<Segment>) {
        // Link segments spanned by `span` from head to tail
        let overlap = Span::new_empty();
        flat.add_links((span.start.index()..span.end.index() - 1).map(|idx| Link {
            from: Handle::new(Id::new(idx), Orientation::Forward),
            to: Handle::new(Id::new(idx + 1), Orientation::Forward),
            overlap,
        }));
    }

    // Add new, chopped segments
    for seg in gfa.segs.all().iter() {
        let len = seg.len();
        if len <= args.c {
            // Leave the segment as is
            let id = flat.segs.add(Segment {
                name: max_node_id,
                seq: seg.seq,
                optional: Span::new_empty(), // TODO: Optional data may stay valid when seg not chopped?
            });
            max_node_id += 1;
            seg_map.push(Span::new(id, flat.segs.next_id()));
        } else {
            let seq_end = seg.seq.end;
            let mut offset = seg.seq.start.index();
            let segs_start = flat.segs.next_id();
            // Could also generate end_id by setting it equal to the start_id and
            // updating it for each segment that is added - only benefits us if we
            // don't unroll the last iteration of this loop
            while offset < seq_end.index() - args.c {
                // Generate a new segment of length c
                flat.segs.add(Segment {
                    name: max_node_id,
                    seq: Span::new(Id::new(offset), Id::new(offset + args.c)),
                    optional: Span::new_empty(),
                });
                offset += args.c;
                max_node_id += 1;
            }
            // Generate the last segment
            flat.segs.add(Segment {
                name: max_node_id,
                seq: Span::new(Id::new(offset), seq_end),
                optional: Span::new_empty(),
            });
            max_node_id += 1;
            let new_seg_span = Span::new(segs_start, flat.segs.next_id());
            seg_map.push(new_seg_span);
            if args.l {
                link_forward(&mut flat, &new_seg_span);
            }
        }
    }

    // For each path, add updated handles. Then add the updated path
    for path in gfa.paths.all().iter() {
        let path_start = flat.steps.next_id();
        let mut path_end = flat.steps.next_id();
        // Generate the new handles
        // Tentative to-do: see if it is faster to read Id from segs than to re-generate it?
        for step in gfa.get_path_steps(path) {
            let range = {
                let span = seg_map[step.segment().index()];
                std::ops::Range::from(span)
            };
            match step.orient() {
                Orientation::Forward => {
                    // In this builder, Id.index() == seg.name - 1 for all seg
                    path_end = flat
                        .add_steps(range.map(|idx| Handle::new(Id::new(idx), Orientation::Forward)))
                        .end;
                }
                Orientation::Backward => {
                    path_end = flat
                        .add_steps(
                            range
                                .rev()
                                .map(|idx| Handle::new(Id::new(idx), Orientation::Backward)),
                        )
                        .end;
                }
            }
        }

        // Add the updated path
        flat.paths.add(Path {
            name: path.name,
            steps: Span::new(path_start, path_end),
            overlaps: Span::new_empty(),
        });
    }

    // If the 'l' flag is specified, compute the links in the new graph
    if args.l {
        // For each link in the old graph, from handle A -> B:
        //      Add a link from
        //          (A.forward ? (A.end, forward) : (A.begin, backwards))
        //          -> (B.forward ? (B.begin, forward) : (B.end ? backwards))

        for link in gfa.links.all().iter() {
            let new_from = {
                let old_from = link.from;
                let chopped_segs = seg_map[old_from.segment().index()];
                let seg_id = match old_from.orient() {
                    Orientation::Forward => chopped_segs.end - 1,
                    Orientation::Backward => chopped_segs.start,
                };
                Handle::new(seg_id, old_from.orient())
            };
            let new_to = {
                let old_to = link.to;
                let chopped_segs = seg_map[old_to.segment().index()];
                let seg_id = match old_to.orient() {
                    Orientation::Forward => chopped_segs.start,
                    Orientation::Backward => chopped_segs.end - 1,
                };
                Handle::new(seg_id, old_to.orient())
            };
            flat.add_link(new_from, new_to, vec![]);
        }
    }

    Ok(flat)
}
