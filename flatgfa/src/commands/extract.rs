use crate::fgfa_ds::flatgfa::{self, Handle, Link, Path, Segment};
use crate::fgfa_ds::pool::{self, Id, Span, Store};
use argh::FromArgs;
use std::collections::HashMap;

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
    #[argh(option, short = 'd', long = "max-distance-subpaths", default = "300000")]
    max_distance_subpaths: usize, // TODO: possibly make this bigger

    /// maximum number of iterations before we stop merging subpaths
    #[argh(option, short = 'e', long = "max-merging-iterations", default = "6")]
    num_iterations: usize // TODO: probably make this smaller
}

pub fn extract(
    gfa: &flatgfa::FlatGFA,
    args: Extract,
) -> Result<flatgfa::HeapGFAStore, &'static str> {
    let origin_seg = gfa.find_seg(args.seg_name).ok_or("segment not found")?;

    let mut subgraph = SubgraphBuilder::new(gfa);
    subgraph.add_header();
    subgraph.extract(origin_seg, args.link_distance, args.max_distance_subpaths, args.num_iterations);
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
    fn include_link(&mut self, link: &Link) {
        let from = self.tr_handle(link.from);
        let to = self.tr_handle(link.to);
        let overlap = self.old.get_alignment(link.overlap);
        self.store.add_link(from, to, overlap.ops.into());
    }

    /// Add a single subpath from the given path to the subgraph.
    fn include_subpath(&mut self, path: &Path, start: &SubpathStart, end_pos: usize) {
        let steps = pool::Span::new(start.step, self.store.steps.next_id()); // why the next id?
        let name = format!("{}:{}-{}", self.old.get_path_name(path), start.pos, end_pos);
        self.store
            .add_path(name.as_bytes(), steps, std::iter::empty());
    }

    /// Identify all the subpaths in a path from the original graph that cross through
    /// segments in this subgraph and merge them if possible.
    fn merge_subpaths(&mut self, path: &Path, max_distance_subpaths: usize) {
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
                    let subpath_span = Span::new(path.steps.start + *start as u32, path.steps.start + idx as u32);
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
    fn find_subpaths(&mut self, path: &Path) {
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
    fn tr_handle(&self, old_handle: Handle) -> Handle {
        // TODO: is this just generating the handle or should we add it to the new graph?
        Handle::new(self.seg_map[&old_handle.segment()], old_handle.orient())
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
    fn extract(&mut self, origin: Id<Segment>, dist: usize, max_distance_subpaths: usize, num_iterations: usize) {
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