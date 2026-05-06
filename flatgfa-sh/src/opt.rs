use crate::ir::{Builder, Instr, Op, Program, ResourceKind};
use std::fs;

fn find_og_pair(instrs: &[Instr]) -> Option<(usize, usize)> {
    // Search for a `parse-gfa` instruction.
    for (parse_idx, instr) in instrs.iter().enumerate() {
        if let Op::ParseGFA = instr.op {
            // Search for an `odgi-view` instruction that produces the GFA.
            let gfa = instr.input;
            if let Some(view_idx) = instrs.iter().position(|instr| match instr.op {
                Op::OdgiView => instr.output == gfa,
                _ => false,
            }) {
                return Some((view_idx, parse_idx));
            }
        }
    }
    None
}

fn opt_og_pair(builder: &mut Builder, view_idx: usize, parse_idx: usize) {
    // Get the input `.og` file to this pair.
    let og_filename = builder.file_name(builder.instrs[view_idx].input);
    let stem = og_filename
        .strip_suffix(".og")
        .expect("odgi-view inputs must end in .og");

    // Get the parsed FlatGFA output resource.
    let old_gfa = builder.instrs[parse_idx].output;

    // Does the FlatGFA exist?
    let flat_filename = format!("{stem}.flatgfa");
    if fs::exists(&flat_filename).unwrap() {
        // Make a new instruction to load the FlatGFA.
        let new_gfa = builder.rsrc(ResourceKind::Mmap);
        let new_instr = Instr {
            input: builder.file(flat_filename),
            output: new_gfa,
            op: Op::MapFile,
        };

        // Swap it in where the old `parse-gfa` used to be, and remove the
        // `odgi-view`.
        builder.instrs[parse_idx] = new_instr;
        builder.instrs.remove(view_idx);

        // Use the new resource in the rest of the program.
        builder.replace_rsrc(old_gfa, new_gfa);
    }
}

pub fn optimize(prog: Program) -> Program {
    let mut builder = Builder::rebuild(prog);

    if let Some((view_idx, parse_idx)) = find_og_pair(&builder.instrs) {
        opt_og_pair(&mut builder, view_idx, parse_idx);
    }

    builder.build()
}
