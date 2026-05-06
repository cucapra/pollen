use crate::ir::{Builder, Instr, Op, Program, ResourceKind};
use std::fs;

fn find_og_pair(instrs: &[Instr]) -> Option<(usize, usize)> {
    // Search for a `parse-gfa` instruction.
    let (parse_idx, parse_instr) = instrs
        .iter()
        .enumerate()
        .find(|(_, instr)| matches!(instr.op, Op::ParseGFA))?;

    // Search for an `odgi-view` instruction that produces the GFA.
    let gfa = parse_instr.input;
    let view_idx = instrs
        .iter()
        .position(|instr| matches!(instr.op, Op::OdgiView) && instr.output == gfa)?;

    Some((view_idx, parse_idx))
}

fn opt_og_pair(builder: &mut Builder, view_idx: usize, parse_idx: usize) {
    // Get the stem of the input `.og` file to this pair.
    let stem = builder
        .file_name(builder.instrs[view_idx].input)
        .strip_suffix(".og")
        .expect("odgi-view inputs must end in .og")
        .to_string();

    // Try replacing this with a FlatGFA load.
    if replace_with_flat(builder, &stem, parse_idx) {
        // Remove the `odgi-view` too.
        builder.instrs.remove(view_idx);
        return;
    }

    // Otherwise, try replacing this with a direct text GFA parse.
    let text_filename = format!("{stem}.gfa");
    if fs::exists(&text_filename).unwrap() {
        // Make the `parse-gfa` read from this file, and remove the `odgi-view`.
        builder.instrs[parse_idx].input = builder.file(text_filename);
        builder.instrs.remove(view_idx);
    }
}

fn opt_gfa_parse(builder: &mut Builder) {
    // Search for a `parse-gfa` instruction.
    if let Some((parse_idx, parse_instr)) = builder
        .instrs
        .iter()
        .enumerate()
        .find(|(_, instr)| matches!(instr.op, Op::ParseGFA))
    {
        // Get the stem of the input `.gfa` file, if it's a file.
        if parse_instr.input.kind != ResourceKind::File {
            return;
        }
        let stem = builder
            .file_name(parse_instr.input)
            .strip_suffix(".gfa")
            .expect("parse-gfa inputs must end in .gfa")
            .to_string();

        // Try replacing it with a FlatGFA load.
        replace_with_flat(builder, &stem, parse_idx);
    }
}

fn replace_with_flat(builder: &mut Builder, stem: &str, instr_idx: usize) -> bool {
    // Does the FlatGFA exist?
    let flat_filename = format!("{stem}.flatgfa");
    if !fs::exists(&flat_filename).unwrap() {
        return false;
    }

    // Make a new instruction to load the FlatGFA.
    let new_gfa = builder.rsrc(ResourceKind::Mmap);
    let new_instr = Instr {
        input: builder.file(flat_filename),
        output: new_gfa,
        op: Op::MapFile,
    };

    // Swap it in where the old producer instruction used to be.
    let old_gfa = builder.instrs[instr_idx].output;
    builder.instrs[instr_idx] = new_instr;

    // Use the new resource in the rest of the program, replacing the producer
    // instruction's old result.
    builder.replace_rsrc(old_gfa, new_gfa);

    true
}

pub fn optimize(prog: Program) -> Program {
    let mut builder = Builder::rebuild(prog);

    // Optimize `parse-gfa` when we have a FlatGFA file.
    opt_gfa_parse(&mut builder);

    // Optimize `odgi-view` when we have a FlatGFA file.
    if let Some((view_idx, parse_idx)) = find_og_pair(&builder.instrs) {
        opt_og_pair(&mut builder, view_idx, parse_idx);
    }

    builder.build()
}
