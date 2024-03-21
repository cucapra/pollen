use crate::flatgfa;
use argh::FromArgs;

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
    }
}
