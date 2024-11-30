use crate::flatgfa;
use crate::memfile::{map_file, MemchrSplit};
use argh::FromArgs;
use bstr::BStr;

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
    let gaf_buf = map_file(&args.gaf);
    for line in MemchrSplit::new(b'\n', &gaf_buf) {
        let mut field_iter = MemchrSplit::new(b'\t', line);
        let read_name = BStr::new(field_iter.next().unwrap());
        dbg!(read_name);

        // Skip the other fields up to the actual path. Would be nice if
        // `Iterator::advance_by` was stable.
        field_iter.next().unwrap();
        field_iter.next().unwrap();
        field_iter.next().unwrap();
        field_iter.next().unwrap();

        // Step through the path. Using MemchrSplit is pretty lazy; it would be
        // better to manage the indices directly.
        let path = field_iter.next().unwrap();
        for byte in path {
            if *byte == b'<' {
                println!("backward!");
            } else if *byte == b'>' {
                println!("forward!");
            }
            break;
        }
    }
}
