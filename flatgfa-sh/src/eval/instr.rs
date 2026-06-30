use super::{Env, Input};
use crate::ir::{Encoding, Instr, Op, Resource, ResourceKind};
use flatgfa::{
    self, HeapGFAStore,
    flatbed::HeapBEDStore,
    memfile,
    ops::{self, window_depth::Windows},
};

/// Execute a single instruction.
pub fn eval(env: &mut Env, instr: &Instr) {
    match &instr.op {
        Op::NodeDepth => node_depth(env, instr.inputs[0], instr.output),
        Op::PathDepth { path } => path_depth(env, instr.inputs[0], instr.output, path),
        Op::PathLength { path } => path_length(env, instr.inputs[0], instr.output, path),
        Op::Exec { command, args } => exec(env, instr.inputs[0], instr.output, command, args),
        Op::ParseGFA => parse_gfa(env, instr.inputs[0], instr.output),
        Op::MapFile => map_file(env, instr.inputs[0], instr.output),
        Op::ParseBED => parse_bed(env, instr.inputs[0], instr.output),
        Op::MakeWindows { size } => make_windows(env, instr.inputs[0], instr.output, *size),
        Op::OdgiView => odgi_view(env, instr.inputs[0], instr.output),
        Op::IntervalDepth => interval_depth(env, instr.inputs[0], instr.inputs[1], instr.output),
        Op::GzipDecompress => gzip_decompress(env, instr.inputs[0], instr.output),
    }
}

fn node_depth(env: &mut Env, input: Resource, output: Resource) {
    let mut out = env.bytes_output(output).expect("bytes output");
    let gfa = env.flatgfa(input);
    let (depths, uniq_depths) = ops::depth::seg_depth_with_uniq(&gfa);
    out.emit(ops::depth::SegDepth {
        gfa: &gfa,
        depths,
        uniq_depths,
    })
    .unwrap();
}

fn path_depth(env: &mut Env, input: Resource, output: Resource, path: &Option<String>) {
    let out = env.bytes_output(output);
    let gfa = env.flatgfa(input);
    if let Some(path_name) = &path {
        // TODO More elegantly handle missing paths.
        let path_id = env
            .flatgfa(input)
            .find_path(path_name.as_bytes().into())
            .expect("no such path found");
        let (lengths, depths) = ops::depth::path_depth(&gfa, std::iter::once(path_id));
        let depth = ops::depth::PathDepth {
            gfa: &gfa,
            depths,
            lengths,
            paths: std::iter::once(path_id),
        };
        match output.kind {
            ResourceKind::BEDStore => env.bed_stores[output] = Some(depth.as_bed()),
            _ => out.expect("bytes output").emit(depth).unwrap(),
        }
    } else {
        let (lengths, depths) = ops::depth::path_depth(&gfa, gfa.paths.ids());
        let depth = ops::depth::PathDepth {
            gfa: &gfa,
            depths,
            lengths,
            paths: gfa.paths.ids(),
        };
        match output.kind {
            ResourceKind::BEDStore => env.bed_stores[output] = Some(depth.as_bed()),
            _ => out.expect("bytes output").emit(depth).unwrap(),
        }
    }
}

fn path_length(env: &mut Env, input: Resource, output: Resource, path_name: &str) {
    let gfa = env.flatgfa(input);
    let path = gfa.find_path(path_name.into()).expect("no such path found");

    // Add up the total length in base pairs.
    let mut len = 0;
    for step in gfa.get_path_steps(&gfa.paths[path]) {
        len += gfa.segs[step.segment()].len();
    }

    // Generate a (very short) BED table.
    let mut store = HeapBEDStore::default();
    store.add_entry(path_name.as_bytes(), 0, len as u64);

    env.bed_stores[output] = Some(store);
}

fn exec(env: &mut Env, input: Resource, output: Resource, command: &String, args: &[String]) {
    env.run_cmd(command, args, input, output);
}

fn parse_gfa(env: &mut Env, input: Resource, output: Resource) {
    use flatgfa::parse::Parser;
    use std::io::BufRead;

    // Handle compressed or uncompressed input streams.
    // TODO: This is currently worrisomely verbose, and it's only going to get
    // worse if we try to apply this to other parsers.
    fn parse_stream<R: BufRead>(stream: R, encoding: Encoding) -> HeapGFAStore {
        use flatgfa::parse::Parser;
        match encoding {
            Encoding::None => Parser::for_heap().parse_stream(stream),
            Encoding::Gzip => {
                #[cfg(feature = "compress")]
                {
                    use flate2::bufread::GzDecoder;
                    use std::io::BufReader;
                    Parser::for_heap().parse_stream(BufReader::new(GzDecoder::new(stream)))
                }

                #[cfg(not(feature = "compress"))]
                panic!("compression not supported")
            }
        }
    }

    // For unencoded files on disk, we can use a (possibly faster) memchr-based
    // parser. Otherwise, use the stream parser.
    let store = match env.bytes_input(input).expect("text input") {
        Input::File(file) => match input.encoding {
            Encoding::None => Parser::for_heap().parse_mem(file.as_ref()),
            _ => parse_stream(file.as_ref(), input.encoding),
        },
        Input::Stdin(stream) => parse_stream(stream, input.encoding),
        Input::Pipe(stream) => parse_stream(stream, input.encoding),
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

    // We do not yet support decompression.
    // TODO: We need to centralize this logic, so instruction implementations
    // like this one cannot easily "forget" to handle compressed streams.
    assert!(matches!(input.encoding, Encoding::None));

    let store = match env.bytes_input(input).expect("text input") {
        Input::File(file) => BEDParser::for_heap().parse_mem(file.as_ref()),
        Input::Stdin(stream) => BEDParser::for_heap().parse_stream(stream),
        Input::Pipe(stream) => BEDParser::for_heap().parse_stream(stream),
    };

    env.bed_stores[output] = Some(store);
}

fn make_windows(env: &mut Env, input: Resource, output: Resource, size: usize) {
    let store = env.bed_stores[input].take().unwrap();
    let in_bed = store.as_ref();

    // Generate a series of windows for each input interval.
    let all_windows = in_bed.entries.all().iter().map(|entry| Windows {
        name: in_bed.get_name_of_entry(entry),
        start: entry.start,
        end: entry.end,
        size: size.try_into().unwrap(),
    });

    // Output as either FlatBED or text.
    match output.kind {
        ResourceKind::BEDStore => {
            let mut out = HeapBEDStore::default();
            for windows in all_windows {
                windows.emit_bed(&mut out);
            }
            env.bed_stores[output] = Some(out);
        }
        _ => {
            let mut out = env.bytes_output(output).expect("bytes output");
            for windows in all_windows {
                out.emit(windows).unwrap();
            }
        }
    }
}

fn odgi_view(env: &mut Env, input: Resource, output: Resource) {
    let og_file = env.file_name(input).to_string();
    env.run_cmd(
        "odgi",
        &["view", "-g", "-i", &og_file],
        Resource::stdin(),
        output,
    )
}

fn interval_depth(env: &mut Env, gfa_rsrc: Resource, bed_rsrc: Resource, output: Resource) {
    let bed_store = env.bed_stores[bed_rsrc].take().unwrap();
    let gfa = env.flatgfa(gfa_rsrc);

    let depths = ops::window_depth::bed_depth(&gfa, &bed_store.as_ref());

    let mut out = env.bytes_output(output).expect("bytes output");
    out.write("#path\tstart\tend\tmean.depth\n").unwrap();
    out.emit(ops::window_depth::IntervalDepth {
        intervals: bed_store.as_ref(),
        depths,
    })
    .unwrap();
}

#[cfg(feature = "compress")]
fn gzip_decompress(env: &mut Env, input: Resource, output: Resource) {
    use flate2::bufread::GzDecoder;

    let mut out = env.bytes_output(output).expect("bytes output");
    match env.bytes_input(input).expect("bytes input") {
        Input::File(file) => out.copy(&mut GzDecoder::new(file.as_ref())),
        Input::Stdin(stream) => out.copy(&mut GzDecoder::new(stream)),
        Input::Pipe(stream) => out.copy(&mut GzDecoder::new(stream)),
    }
    .expect("decompression failed");
}

#[cfg(not(feature = "compress"))]
fn gzip_decompress(env: &mut Env, input: Resource, output: Resource) {
    env.run_cmd("gzip", &["-d"], input, output);
}
