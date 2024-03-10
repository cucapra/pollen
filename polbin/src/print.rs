use crate::flatgfa;
use std::fmt;

impl fmt::Display for flatgfa::Orientation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            flatgfa::Orientation::Forward => write!(f, "+"),
            flatgfa::Orientation::Backward => write!(f, "-"),
        }
    }
}

impl fmt::Display for flatgfa::AlignOpcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            flatgfa::AlignOpcode::Match => write!(f, "M"),
            flatgfa::AlignOpcode::Gap => write!(f, "N"),
            flatgfa::AlignOpcode::Insertion => write!(f, "D"),
            flatgfa::AlignOpcode::Deletion => write!(f, "I"),
        }
    }
}

impl<'a> fmt::Display for flatgfa::Alignment<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for op in self.ops {
            write!(f, "{}{}", op.len, op.op)?;
        }
        Ok(())
    }
}

fn print_step(gfa: &flatgfa::FlatGFA, handle: &flatgfa::Handle) {
    let seg = &gfa.segs[handle.segment];
    print!("{}{}", seg.name, handle.orient);
}

fn print_path(gfa: &flatgfa::FlatGFA, path: &flatgfa::PathInfo) {
    print!("P\t{}\t", gfa.get_path_name(path));
    let steps = gfa.get_steps(path);
    print_step(&gfa, &steps[0]);
    for step in steps[1..].iter() {
        print!(",");
        print_step(&gfa, step);
    }
    print!("\t");
    let overlaps = gfa.get_overlaps(path);
    if overlaps.is_empty() {
        print!("*");
    } else {
        print!("{}", gfa.get_alignment(&overlaps[0]));
        for overlap in overlaps[1..].iter() {
            print!(",{}", gfa.get_alignment(overlap));
        }
    }
    println!();
}

fn print_link(gfa: &flatgfa::FlatGFA, link: &flatgfa::LinkInfo) {
    println!(
        "L\t{}\t{}\t{}\t{}\t{}",
        gfa.segs[link.from.segment].name,
        link.from.orient,
        gfa.segs[link.to.segment].name,
        link.to.orient,
        gfa.get_alignment(&link.overlap)
    );
}

/// Print our flat representation as a GFA text file to stdout.
pub fn print(gfa: &flatgfa::FlatGFA) {
    match &gfa.header {
        Some(version) => println!("H\tVN:Z:{}", version),
        None => {}
    }
    for seg in &gfa.segs {
        println!("S\t{}\t{}", seg.name, gfa.get_seq(seg));
    }
    for path in &gfa.paths {
        print_path(gfa, path);
    }
    for link in &gfa.links {
        print_link(gfa, link);
    }
}
