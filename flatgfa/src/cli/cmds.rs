use crate::flatgfa::{self, Segment};
use crate::memfile::map_file;
use crate::namemap::NameMap;
use crate::ops;
use crate::packedseq::{self, PackedSeqStore};
use crate::pool::Id;
use argh::FromArgs;
use rayon::iter::ParallelIterator;
use std::collections::HashMap;
use std::io::Read;

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
        println!("{}", ops::bench::line_count(&filename, args.parallel));
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
            uniq_paths[id.index()],
        );
    }
}

/// chop the segments in a graph into sizes of N or smaller
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "chop")]
pub struct Chop {
    /// maximimum segment size
    #[argh(option, short = 'c')]
    count: usize,

    /// compute new links
    #[argh(switch, short = 'l')]
    links: bool,
}

/// Chop a graph into segments of size no larger than c
/// By default, compact node ids
/// CIGAR strings, links, and optional Segment data are invalidated by chop
/// Generates a new graph, rather than modifying the old one in place
pub fn chop<'a>(
    gfa: &'a flatgfa::FlatGFA<'a>,
    args: Chop,
) -> Result<flatgfa::HeapGFAStore, &'static str> {
    Ok(ops::chop::chop(gfa, args.count, args.links))
}

/// look up positions from a GAF file
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "gaf")]
pub struct GAFLookup {
    /// GAF file associated with the GFA
    #[argh(positional)]
    gaf: String,

    /// print the actual sequences
    #[argh(switch, short = 's')]
    seqs: bool,

    /// benchmark only: print nothing; limit reads if nonzero
    #[argh(switch, short = 'b')]
    bench: bool,

    /// parallelize the GAF parser
    #[argh(switch, short = 'p')]
    parallel: bool,
}

pub fn gaf_lookup(gfa: &flatgfa::FlatGFA, args: GAFLookup) {
    // Build a map to efficiently look up segments by name.
    let name_map = NameMap::build(gfa);

    let gaf_buf = map_file(&args.gaf);
    let parser = ops::gaf::GAFParser::new(&gaf_buf);

    if args.parallel {
        if args.bench {
            let count = ParallelIterator::map(parser, |read| {
                ops::gaf::PathChunker::new(gfa, &name_map, read).count()
            })
            .reduce(|| 0, |a, b| a + b);
            println!("{}", count);
        } else {
            unimplemented!("only the no-op mode is parallel")
        }
    } else if args.seqs {
        // Print the actual sequences for each chunk in the GAF.
        for read in parser {
            print!("{}\t", read.name);
            for event in ops::gaf::PathChunker::new(gfa, &name_map, read) {
                event.print_seq(gfa);
            }
            println!();
        }
    } else if args.bench {
        // Benchmarking mode: just process all the chunks but print nothing.
        let mut count = 0;
        for read in parser {
            for _event in ops::gaf::PathChunker::new(gfa, &name_map, read) {
                count += 1;
            }
        }
        println!("{}", count);
    } else {
        // Just print some info about the offsets in the segments.
        for read in parser {
            println!("{}", read.name);
            for event in ops::gaf::PathChunker::new(gfa, &name_map, read) {
                event.print(gfa);
            }
        }
    }
}

// ///
// #[derive(FromArgs, PartialEq, Debug)]
// #[argh(subcommand, name = "seq-import")]
// pub struct SeqImport {
//     #[argh(positional)]
//     filename: String,
// }

// pub fn seq_import(args: SeqImport) {
//     let vec = packedseq::import(&args.filename);
//     let _ = std::fs::remove_file(&args.filename);
//     let store = PackedSeqStore::create(vec);
//     let view = store.as_ref();
//     print!("{}", view);
// }

// ///
// #[derive(FromArgs, PartialEq, Debug)]
// #[argh(subcommand, name = "seq-export")]
// pub struct SeqExport {
//     #[argh(positional)]
//     filename: String,
// }

// pub fn seq_export(args: SeqExport) {
//     let mut input = String::new();
//     std::io::stdin()
//         .read_to_string(&mut input)
//         .expect("Stdin read failure");

//     println!("Input read: {:?}", input);

//     let vec: Vec<packedseq::Nucleotide> = input
//         .chars()
//         .filter(|c| !c.is_whitespace())
//         .map(|c| match c {
//             'A' => packedseq::Nucleotide::A,
//             'C' => packedseq::Nucleotide::C,
//             'T' => packedseq::Nucleotide::T,
//             'G' => packedseq::Nucleotide::G,
//             _ => panic!("Invalid character in fle: {}", c),
//         })
//         .collect();

//     let store = packedseq::PackedSeqStore::create(vec);
//     let view = store.as_ref();
//     packedseq::export(view, &args.filename);
// }
