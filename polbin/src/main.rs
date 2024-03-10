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
        println!();
    }
    for link in &gfa.links {
        print!("L\t{}\t", gfa.segs[link.from.segment].name);
        print_orient(&link.from.orient);
        print!("\t{}\t", gfa.segs[link.to.segment].name);
        print_orient(&link.to.orient);
        println!();
    }
}
