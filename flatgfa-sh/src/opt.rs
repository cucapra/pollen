use crate::ir::{Builder, Instr, MapFileInstr, Program};
use std::fs;

fn find_og_pair(instrs: &[Instr]) -> Option<(usize, usize)> {
    // Search for a `parse-gfa` instruction.
    for (parse_idx, instr) in instrs.iter().enumerate() {
        if let Instr::ParseGFA(parse) = instr {
            // Search for an `odgi-view` instruction that produces the GFA.
            let gfa = parse.input;
            if let Some((view_idx, Instr::OdgiView(view))) =
                instrs.iter().enumerate().find(|(_, instr)| match instr {
                    Instr::OdgiView(view) => view.output == gfa,
                    _ => false,
                })
            {
                dbg!(parse_idx, parse, view_idx, view);
                return Some((view_idx, parse_idx));
            }
        }
    }
    None
}

pub fn optimize(prog: Program) -> Program {
    let mut builder = Builder::rebuild(prog);

    if let Some((view_idx, parse_idx)) = find_og_pair(&builder.instrs) {
        dbg!(view_idx, parse_idx);

        // Get the input `.og` file to this pair.
        let og_filename = if let Instr::OdgiView(view) = &builder.instrs[view_idx] {
            builder.file_name(view.input)
        } else {
            panic!()
        };
        let stem = og_filename
            .strip_suffix(".og")
            .expect("odgi-view inputs must end in .og");

        // Get the final FlatGFA output resource.
        let gfa_rsrc = if let Instr::ParseGFA(parse) = &builder.instrs[parse_idx] {
            parse.output
        } else {
            panic!()
        };

        // Does the FlatGFA exist?
        let flat_filename = format!("{stem}.flatgfa");
        if fs::exists(&flat_filename).unwrap() {
            // Make a new instruction to load the FlatGFA.
            let flat_file = builder.file(flat_filename);
            builder.add_instr(Instr::MapFile(MapFileInstr {
                input: flat_file,
                output: gfa_rsrc,
            }));
        }
    }

    builder.build()
}
