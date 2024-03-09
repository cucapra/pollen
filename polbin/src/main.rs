mod flatgfa;
mod parse;

fn print_step(handle: &flatgfa::Handle) {
    print!("{}", handle.segment);
    if handle.forward {
        print!("+");
    } else {
        print!("-");
    }
}

fn main() {
    let stdin = std::io::stdin();
    let flat = parse::parse(stdin.lock());

    for seg in &flat.segs {
        println!("S\t{}\t{}", seg.name, flat.get_seq(seg));
    }
    for path in &flat.paths {
        print!("P\t{}\t", path.name);
        let steps = flat.get_steps(path);
        print_step(&steps[0]);
        for step in steps[1..].iter() {
            print!(",");
            print_step(step);
        }
        println!();
    }
}
