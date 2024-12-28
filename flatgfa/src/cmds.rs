use crate::flatgfa::{self, Handle, Link, Orientation, Path, Segment};
use crate::memfile;
use crate::ops;
use crate::pool::{Id, Span, Store};
use crate::{GFAStore, HeapFamily};
use argh::FromArgs;
use rayon::iter::ParallelIterator;
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

    // Print the match.
    let found = ops::position::position(gfa, path, offset);
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

    /// enable parallelism when available
    #[argh(switch, short = 'p')]
    parallel: bool,
}

pub fn bench(args: Bench) {
    // TODO: We don't need a GFA for (some of) these? So avoid opening it.
    if let Some(filename) = args.wcl {
        let buf = memfile::map_file(&filename);
        let split = memfile::MemchrSplit::new(b'\n', &buf);
        let count = if args.parallel {
            ParallelIterator::count(split)
        } else {
            Iterator::count(split)
        };
        println!("{}", count);
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

    let mut subgraph = ops::extract::SubgraphBuilder::new(gfa);
    subgraph.add_header();
    subgraph.extract(
        origin_seg,
        args.link_distance,
        args.max_distance_subpaths,
        args.num_iterations,
    );
    Ok(subgraph.store)
}

/// compute node depth, the number of times paths cross a node
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "depth")]
pub struct Depth {}

pub fn depth(gfa: &flatgfa::FlatGFA) {
    let (depths, uniq_paths) = ops::depth::depth(gfa);

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
