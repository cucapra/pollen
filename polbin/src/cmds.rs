use crate::flatgfa;
use crate::pool::Index;
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
