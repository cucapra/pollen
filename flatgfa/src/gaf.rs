use crate::flatgfa;
use crate::memfile::{map_file, MemchrSplit};
use argh::FromArgs;

/// look up positions from a GAF file
#[derive(FromArgs, PartialEq, Debug)]
#[argh(subcommand, name = "gaf")]
pub struct GafLookup {
    /// path_name,offset,orientation
    #[argh(positional)]
    gaf: String,
}

pub fn gaf_lookup(gfa: &flatgfa::FlatGFA, args: GafLookup) {
    // Read the GAF file, I suppose.
    // let file = File::open(args.gaf).unwrap();
    // for line in io::BufReader::new(file).lines() {}
    let gaf_buf = map_file(&args.gaf);
    for line in MemchrSplit::new(b'\n', &gaf_buf) {
        println!("line {}", line.len());
    }
}
