use std::fmt;

mod flatgfa;
mod parse;

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

fn print_step(gfa: &flatgfa::FlatGFA, handle: &flatgfa::Handle) {
    let seg = &gfa.segs[handle.segment];
    print!("{}{}", seg.name, handle.orient);
}

fn print_cigar(ops: &[flatgfa::AlignOp]) {
    for op in ops {
        print!("{}{}", op.len, op.op);
    }
}

fn main() {
    let stdin = std::io::stdin();
    let gfa = parse::Parser::parse(stdin.lock());

    match &gfa.header {
        Some(version) => println!("H\tVN:Z:{}", version),
        None => {}
    }
    for seg in &gfa.segs {
        println!("S\t{}\t{}", seg.name, gfa.get_seq(seg));
    }
    for path in &gfa.paths {
        print!("P\t{}\t", path.name);
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
            print_cigar(gfa.get_alignment(&overlaps[0]));
            for overlap in overlaps[1..].iter() {
                print!(",");
                print_cigar(gfa.get_alignment(overlap));
            }
        }
        println!();
    }
    for link in &gfa.links {
        print!(
            "L\t{}\t{}\t{}\t{}\t",
            gfa.segs[link.from.segment].name,
            link.from.orient,
            gfa.segs[link.to.segment].name,
            link.to.orient
        );
        print_cigar(gfa.get_alignment(&link.overlap));
        println!();
    }
}
