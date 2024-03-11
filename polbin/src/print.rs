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
            write!(f, "{}{}", op.len(), op.op())?;
        }
        Ok(())
    }
}

fn print_step(gfa: &flatgfa::FlatGFA, handle: &flatgfa::Handle) {
    let seg = &gfa.segs[handle.segment()];
    print!("{}{}", seg.name, handle.orient());
}

fn print_path(gfa: &flatgfa::FlatGFA, path: &flatgfa::Path) {
    print!("P\t{}\t", gfa.get_path_name(path));
    let steps = gfa.get_steps(path);
    print_step(gfa, &steps[0]);
    for step in steps[1..].iter() {
        print!(",");
        print_step(gfa, step);
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

fn print_link(gfa: &flatgfa::FlatGFA, link: &flatgfa::Link) {
    println!(
        "L\t{}\t{}\t{}\t{}\t{}",
        gfa.segs[link.from.segment()].name,
        link.from.orient(),
        gfa.segs[link.to.segment()].name,
        link.to.orient(),
        gfa.get_alignment(&link.overlap)
    );
}

fn print_seg(gfa: &flatgfa::FlatGFA, seg: &flatgfa::Segment) {
    print!("S\t{}\t{}", seg.name, gfa.get_seq(seg));
    if !seg.optional.is_empty() {
        print!("\t{}", gfa.get_optional_data(seg));
    }
    println!();
}

/// Print our flat representation as a GFA text file to stdout.
pub fn print(gfa: &flatgfa::FlatGFA) {
    let mut seg_iter = gfa.segs.iter();
    let mut path_iter = gfa.paths.iter();
    let mut link_iter = gfa.links.iter();
    for kind in gfa.get_line_order() {
        match kind {
            flatgfa::LineKind::Header => {
                let version = gfa.header;
                assert!(!version.is_empty());
                println!("H\tVN:Z:{}", version);
            }
            flatgfa::LineKind::Segment => {
                print_seg(gfa, seg_iter.next().expect("too few segments"));
            }
            flatgfa::LineKind::Path => {
                print_path(gfa, path_iter.next().expect("too few paths"));
            }
            flatgfa::LineKind::Link => {
                print_link(gfa, link_iter.next().expect("too few links"));
            }
        }
    }
}
