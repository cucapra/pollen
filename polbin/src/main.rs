mod flatgfa;
mod parse;

fn print_step(gfa: &flatgfa::FlatGFA, handle: &flatgfa::Handle) {
    let seg = &gfa.segs[handle.segment];
    print!("{}", seg.name);
    if handle.forward {
        print!("+");
    } else {
        print!("-");
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
        println!();
    }
    for link in &gfa.links {
        println!(
            "L\t{}\t{}\t{}\t{}",
            gfa.segs[link.from.segment].name,
            if link.from.forward { '+' } else { '-' },
            gfa.segs[link.to.segment].name,
            if link.to.forward { '+' } else { '-' },
        );
    }
}
