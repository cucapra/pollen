use super::{Env, Input};
use crate::ir::{Instr, Op, Resource, ResourceKind};
use bstr::BStr;
use flatgfa::{self, emit::Emit, memfile, ops};
use std::io;

/// Execute a single instruction.
pub fn eval(env: &mut Env, instr: &Instr) {
    match &instr.op {
        Op::NodeDepth => node_depth(env, instr.input, instr.output),
        Op::PathDepth { path } => path_depth(env, instr.input, instr.output, path),
        Op::Exec { command, args } => exec(env, instr.input, instr.output, command, args),
        Op::ParseGFA => parse_gfa(env, instr.input, instr.output),
        Op::MapFile => map_file(env, instr.input, instr.output),
        Op::ParseBED => parse_bed(env, instr.input, instr.output),
        Op::MakeWindows { size } => make_windows(env, instr.input, instr.output, *size),
        Op::OdgiView => odgi_view(env, instr.input, instr.output),
    }
}

fn node_depth(env: &mut Env, input: Resource, output: Resource) {
    let mut out = env.output(output);
    let gfa = env.flatgfa(input);
    let (depths, uniq_depths) = ops::depth::seg_depth(&gfa);
    out.emit(ops::depth::SegDepth {
        gfa: &gfa,
        depths,
        uniq_depths,
    })
    .unwrap();
}

fn path_depth(env: &mut Env, input: Resource, output: Resource, path: &Option<String>) {
    let mut out = env.output(output);
    let gfa = env.flatgfa(input);
    if let Some(path_name) = &path {
        // TODO More elegantly handle missing paths.
        let path_id = gfa
            .find_path(path_name.as_bytes().into())
            .expect("no such path found");
        let (lengths, depths) = ops::depth::path_depth(&gfa, std::iter::once(path_id));
        out.emit(ops::depth::PathDepth {
            gfa: &gfa,
            depths,
            lengths,
            paths: std::iter::once(path_id),
        })
        .unwrap();
    } else {
        let (lengths, depths) = ops::depth::path_depth(&gfa, gfa.paths.ids());
        out.emit(ops::depth::PathDepth {
            gfa: &gfa,
            depths,
            lengths,
            paths: gfa.paths.ids(),
        })
        .unwrap();
    }
}

fn exec(env: &mut Env, input: Resource, output: Resource, command: &String, args: &[String]) {
    env.run_cmd(command, args, input, output);
}

fn parse_gfa(env: &mut Env, input: Resource, output: Resource) {
    use flatgfa::parse::Parser;

    let store = match env.input(input) {
        Input::File(file) => Parser::for_heap().parse_mem(file.as_ref()),
        Input::Stdin(stream) => Parser::for_heap().parse_stream(stream),
        Input::Pipe(stream) => Parser::for_heap().parse_stream(stream),
    };

    env.gfa_stores[output] = Some(store);
}

fn map_file(env: &mut Env, input: Resource, output: Resource) {
    if let ResourceKind::File = input.kind {
        let mmap = memfile::map_file(env.file_name(input));
        env.mmaps[output] = Some(mmap);
    } else {
        panic!("can only map actual files");
    }
}

fn parse_bed(env: &mut Env, input: Resource, output: Resource) {
    use flatgfa::flatbed::BEDParser;

    let store = match env.input(input) {
        Input::File(file) => BEDParser::for_heap().parse_mem(file.as_ref()),
        Input::Stdin(stream) => BEDParser::for_heap().parse_stream(stream),
        Input::Pipe(stream) => BEDParser::for_heap().parse_stream(stream),
    };

    env.bed_stores[output] = Some(store);
}

struct WindowsBed<'a> {
    name: &'a BStr,
    start: u64,
    end: u64,
    size: u64,
}

impl<'a> Emit for WindowsBed<'a> {
    fn emit(self, f: &mut impl std::io::Write) -> io::Result<()> {
        let mut pos = self.start;
        while pos < self.end {
            let end = (pos + self.size).min(self.end);
            writeln!(f, "{}\t{}\t{}", self.name, pos, end)?;
            pos = end;
        }
        Ok(())
    }
}

fn make_windows(env: &mut Env, input: Resource, output: Resource, size: usize) {
    let store = env.bed_stores[input].take().unwrap();
    let bed = store.as_ref();
    let mut out = env.output(output);
    for entry in bed.entries.all() {
        out.emit(WindowsBed {
            name: bed.get_name_of_entry(entry),
            start: entry.start,
            end: entry.end,
            size: size.try_into().unwrap(),
        })
        .unwrap();
    }
}

fn odgi_view(env: &mut Env, input: Resource, output: Resource) {
    let og_file = env.file_name(input).to_string();
    env.run_cmd(
        "odgi",
        &["view", "-g", "-i", &og_file],
        Resource {
            kind: ResourceKind::Stdin,
            index: 0,
        },
        output,
    )
}
