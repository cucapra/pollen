use std::env;

use flatgfa::{
    memfile::{self, *},
    packedseq::{self, *},
};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 && args[1] == "import" {
        let mmap = memfile::map_file(&args[2]); // args[2] is the filename
        let seq = packedseq::view(&mmap);
        println!("Sequence: {}", seq);
    } else if args.len() > 2 && args[1] == "export" {
        let vec = PackedSeqStore::create(vec![
            Nucleotide::A,
            Nucleotide::C,
            Nucleotide::T,
            Nucleotide::G,
        ]);
        let input = vec.as_ref();
        let filename = &args[2];
        let num_bytes = total_bytes(4);
        let mut mem = map_new_file(filename, num_bytes as u64);
        let mut buf = vec![0u8; num_bytes];
        let buf_ref = &mut buf;
        dump(&input, buf_ref);
        mem[..buf_ref.len()].copy_from_slice(buf_ref);
    } else {
        println!("Incorrect commands provided!");
    }
}
