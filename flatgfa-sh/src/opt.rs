use crate::ir::{Builder, Instr, MapFileInstr, Program, ResourceKind};
use std::fs;

fn find_og_pair(instrs: &[Instr]) -> Option<(usize, usize)> {
    // Search for a `parse-gfa` instruction.
    for (parse_idx, instr) in instrs.iter().enumerate() {
        if let Instr::ParseGFA(parse) = instr {
            // Search for an `odgi-view` instruction that produces the GFA.
            let gfa = parse.input;
            if let Some(view_idx) = instrs.iter().position(|instr| match instr {
                Instr::OdgiView(view) => view.output == gfa,
                _ => false,
            }) {
                return Some((view_idx, parse_idx));
            }
        }
    }
    None
}

pub fn optimize(prog: Program) -> Program {
    let mut builder = Builder::rebuild(prog);

    if let Some((view_idx, parse_idx)) = find_og_pair(&builder.instrs) {
        // Get the input `.og` file to this pair.
        let og_filename = if let Instr::OdgiView(view) = &builder.instrs[view_idx] {
            builder.file_name(view.input)
        } else {
            panic!()
        };
        let stem = og_filename
            .strip_suffix(".og")
            .expect("odgi-view inputs must end in .og");

        // Get the parsed FlatGFA output resource.
        let old_gfa = if let Instr::ParseGFA(parse) = &builder.instrs[parse_idx] {
            parse.output
        } else {
            panic!()
        };

        // Does the FlatGFA exist?
        let flat_filename = format!("{stem}.flatgfa");
        if fs::exists(&flat_filename).unwrap() {
            // Make a new instruction to load the FlatGFA.
            let new_gfa = builder.add_rsrc(ResourceKind::Mmap);
            let new_instr = MapFileInstr {
                input: builder.file(flat_filename),
                output: new_gfa,
            };

            // Swap it in where the old `parse-gfa` used to be, and remove the
            // `odgi-view`.
            builder.instrs[parse_idx] = new_instr;
            builder.instrs.remove(view_idx);

            // Replace all uses of `old_gfa` with `new_gfa`.
            // TODO
        }
    }

    builder.build()
}
