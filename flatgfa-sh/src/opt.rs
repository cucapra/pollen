use crate::ir::{Builder, Instr, Op, Program, Resource, ResourceKind};
use smallvec::SmallVec;
use std::{collections::HashMap, fs};

/// Apply all our optimizations to a program.
pub fn optimize(prog: Program) -> Program {
    let mut builder = Builder::rebuild(prog);

    // Optimize `odgi-view` or `parse-gfa` into `map-file` when a FlatGFA file
    // is available.
    opt_gfa_parse(&mut builder);
    opt_og_parse(&mut builder);
    skip_bed_files(&mut builder);

    builder.build()
}

/// Try to replace odgi inputs with FlatGFA binary inputs.
///
/// Search for an `odgi-view` instruction that reads an `.og` file and feeds
/// into a `gfa-parse` instruction, and (if found) replace that pair with a
/// `map-file` of a similarly-named `.fgfa` file that exists on the filesystem.
fn opt_og_parse(builder: &mut Builder) {
    // Find an `odgi-view` -> `gfa-parse` pair.
    let Some((view_idx, parse_idx)) = find_og_pair(&builder.instrs) else {
        return;
    };

    // Get the stem of the input `.og` file to this pair.
    let stem = builder
        .file_name(builder.instrs[view_idx].inputs[0])
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
        builder.instrs[parse_idx].inputs[0] = builder.file(text_filename);
        builder.instrs.remove(view_idx);
    }
}

fn find_og_pair(instrs: &[Instr]) -> Option<(usize, usize)> {
    // Search for a `parse-gfa` instruction.
    let (parse_idx, parse_instr) = instrs
        .iter()
        .enumerate()
        .find(|(_, instr)| matches!(instr.op, Op::ParseGFA))?;

    // Search for an `odgi-view` instruction that produces the GFA.
    let gfa = parse_instr.inputs[0];
    let view_idx = instrs
        .iter()
        .position(|instr| matches!(instr.op, Op::OdgiView) && instr.output == gfa)?;

    Some((view_idx, parse_idx))
}

/// Optimize GFA file input to FlatGFA input.
///
/// Search for a `parse-gfa` instruction for a `.gfa` file and, when the
/// relevant file exists, replace it with a `map-file` of an equivalent
/// `.flatgfa` file.
fn opt_gfa_parse(builder: &mut Builder) {
    // Search for a `parse-gfa` instruction.
    if let Some((parse_idx, parse_instr)) = builder
        .instrs
        .iter()
        .enumerate()
        .find(|(_, instr)| matches!(instr.op, Op::ParseGFA))
    {
        // Get the stem of the input `.gfa` file, if it's a file.
        if parse_instr.inputs[0].kind != ResourceKind::File {
            return;
        }
        let stem = builder
            .file_name(parse_instr.inputs[0])
            .strip_suffix(".gfa")
            .expect("parse-gfa inputs must end in .gfa")
            .to_string();

        // Try replacing it with a FlatGFA load.
        replace_with_flat(builder, &stem, parse_idx);
    }
}

/// Replace an instruction with a `map-file` to open a FlatGFA binary file.
///
/// If the file `{stem}.flatgfa` exists, replace the instruction at `instr_idx`
/// with a new instruction that maps that file. Update all consuming
/// instructions to use the resulting resource instead of the parsed GFA.
fn replace_with_flat(builder: &mut Builder, stem: &str, instr_idx: usize) -> bool {
    // Does the FlatGFA exist?
    let flat_filename = format!("{stem}.flatgfa");
    if !fs::exists(&flat_filename).unwrap() {
        return false;
    }

    // Make a new instruction to load the FlatGFA.
    let new_gfa = builder.rsrc(ResourceKind::Mmap);
    let new_instr = Instr {
        inputs: SmallVec::from_slice(&[builder.file(flat_filename)]),
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

#[derive(Debug)]
struct DefUse {
    /// The defining instruction for each use.
    ///
    /// For each instruction, this contains a vector that is the same length as
    /// that instruction's `inputs`. Each entry in that vector references (via
    /// index) the instruction that defined that input.
    defs: Vec<SmallVec<[Option<usize>; 2]>>,

    /// The uses of the resource defined by each instruction.
    ///
    /// Each instruction defines the resource that is its `output` resource.
    /// For each instruction, this contains a vector referencing (via index) all
    /// the instructions that use that defined resource. A use occurs when the
    /// resource appears in an instruction's `inputs` before it is overwritten.
    uses: Vec<SmallVec<[usize; 2]>>,
}

fn def_use(instrs: &[Instr]) -> DefUse {
    let mut defs = Vec::with_capacity(instrs.len());
    let mut uses = vec![SmallVec::new(); instrs.len()];
    let mut last_def: HashMap<Resource, usize> = HashMap::new();

    for (idx, instr) in instrs.iter().enumerate() {
        // Find the definition for each use.
        defs.push(
            instr
                .inputs
                .iter()
                .map(|input| last_def.get(input).copied())
                .collect(),
        );

        // Attribute each use to its definition.
        for input in &instr.inputs {
            if let Some(&def_idx) = last_def.get(input) {
                uses[def_idx].push(idx);
            }
        }

        // Record the definition.
        last_def.insert(instr.output, idx);
    }

    DefUse { defs, uses }
}

/// Optimize BED file round trips.
///
/// Search for this pattern:
///
///     something -> "file.bed"
///     parse-bed("file.bed") -> bed-store-0
///
/// And attempt to replace it with:
///
///     something -> bed-store 0
fn skip_bed_files(builder: &mut Builder) {
    // Find def/use pairs consisting of something that produces a BED file and
    // then a `parse-bed` instruction.
    let du = def_use(&builder.instrs);
    let pairs: Vec<_> = builder
        .instrs
        .iter()
        .enumerate()
        .filter_map(|(parse_idx, parse_instr)| {
            // Start with a `parse-bed` instruction.
            let Op::ParseBED = parse_instr.op else {
                return None;
            };

            // Find the instruction that produces this file. If there are no
            // other uses of this file, we can optimize.
            let def_idx = du.defs[parse_idx][0]?;
            if du.uses[def_idx].len() != 1 {
                return None;
            }

            // We match if this is in an allowlist of operations that
            // can either produce BED text files *or* in-memory FlatBED
            // resources.
            let Op::MakeWindows { size: _ } = builder.instrs[def_idx].op else {
                return None;
            };

            Some((def_idx, parse_idx))
        })
        .collect();

    // TODO second+ iterations break because we will change indices :(
    for (def_idx, parse_idx) in pairs {
        // Make the defining instruction produce the parsed FlatBED resource directly.
        builder.instrs[def_idx].output = builder.instrs[parse_idx].output;

        // Delete the parse instruction.
        builder.instrs.remove(parse_idx);
    }
}
