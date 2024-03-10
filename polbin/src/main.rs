mod flatgfa;
mod parse;

fn print_orient(orient: &flatgfa::Orientation) {
    match orient {
        flatgfa::Orientation::Forward => print!("+"),
        flatgfa::Orientation::Backward => print!("-"),
    }
}

fn print_step(gfa: &flatgfa::FlatGFA, handle: &flatgfa::Handle) {
    let seg = &gfa.segs[handle.segment];
    print!("{}", seg.name);
    print_orient(&handle.orient);
}

fn print_cigar(ops: &[flatgfa::AlignOp]) {
    for op in ops {
        print!("{}", op.len);
        match op.op {
            flatgfa::AlignOpcode::Match => print!("M"),
            flatgfa::AlignOpcode::Gap => print!("N"),
            flatgfa::AlignOpcode::Insertion => print!("D"),
            flatgfa::AlignOpcode::Deletion => print!("I"),
        }
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
        print!("L\t{}\t", gfa.segs[link.from.segment].name);
        print_orient(&link.from.orient);
        print!("\t{}\t", gfa.segs[link.to.segment].name);
        print_orient(&link.to.orient);
        print!("\t");
        print_cigar(gfa.get_alignment(&link.overlap));
        println!();
    }
}
