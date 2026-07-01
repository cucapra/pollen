use crate::builder::Builder;
use crate::ir::{self, Op, Resource};
use brush_parser::{
    ast::{
        Command, CommandPrefixOrSuffixItem, CompoundListItem, IoFileRedirectKind,
        IoFileRedirectTarget, IoRedirect, Pipeline, Program, SeparatorOperator, Word,
    },
    word::{WordPiece, WordPieceWithSource},
};
use pico_args::Arguments;

pub fn parse_sh(input: &str) -> Program {
    use brush_parser::{Parser, ParserOptions};
    use std::io::BufReader;

    let opts = ParserOptions::default();
    let buf_reader = BufReader::new(input.as_bytes());
    let mut parser = Parser::new(buf_reader, &opts);
    parser.parse_program().unwrap()
}

/// Translate a single shell command (i.e., an argv list, possibly with stream
/// redirections) to IR instructions.
fn translate_command(
    builder: &mut Builder,
    name: String,
    args: Vec<String>,
    redirects: Vec<IoRedirect>,
    input: Resource,
    output: Resource,
) {
    // Do the input or output come from stream redirections?
    let mut input = input;
    let mut output = output;
    for redirect in redirects {
        match redirect {
            IoRedirect::File(_, kind, target) => {
                let filename = match target {
                    IoFileRedirectTarget::Filename(w) => word_str(w),
                    _ => unimplemented!(),
                };
                match kind {
                    IoFileRedirectKind::Read => input = builder.file(filename),
                    IoFileRedirectKind::Write => output = builder.file(filename),
                    _ => unimplemented!(),
                }
            }
            _ => unimplemented!(),
        }
    }

    // Look for known commands.
    if name == "odgi" {
        translate_odgi(builder, args, input, output);
    } else if name == "bedtools" {
        translate_bedtools(builder, args, input, output);
    } else if name == "gunzip" {
        assert!(args.is_empty(), "no gunzip arguments are supported");
        builder.instr(&[input], output, Op::GzipDecompress);
    } else {
        // Any other command is a "passthrough" shell command.
        builder.instr(
            &[input],
            output,
            Op::Exec {
                command: name,
                args: args.into(),
            },
        );
    }
}

/// Translate an `odgi` subcommand invocation.
fn translate_odgi(builder: &mut Builder, args: Vec<String>, input: Resource, output: Resource) {
    let mut argp = Arguments::from_vec(args.into_iter().map(|s| s.into()).collect());

    // Most odgi commands share an `-i` flag for overriding the input.
    let input = if let Some(filename) = argp.opt_value_from_str(["-i", "--input"]).unwrap() {
        builder.file(filename)
    } else {
        input
    };

    // Get a FlatGFA data structure, either by parsing GFA text or by
    // memory-mapping a FlatGFA binary file.
    let input = builder.load_gfa(input);

    match argp.subcommand().unwrap().as_deref() {
        Some("depth") => {
            // In the `odgi depth` command line, the default is a per-path
            // table. `-d` switches to a per-node table, and `-b` takes a BED
            // file and switches to an interval table.
            if argp.contains("-d") {
                builder.instr(&[input], output, Op::NodeDepth);
            } else if let Some(bed_file) = argp.opt_value_from_str("-b").unwrap() {
                let bed_file = builder.file(bed_file);
                let bed_rsrc = builder.load_bed(bed_file);
                builder.instr(&[input, bed_rsrc], output, Op::IntervalDepth)
            } else {
                builder.instr(
                    &[input],
                    output,
                    Op::PathDepth {
                        path: argp.opt_value_from_str("-r").unwrap(),
                    },
                );
            };
        }
        _ => unimplemented!("unsupported odgi subcommand"),
    }
}

/// Translate a `bedtools` subcommand invocation.
fn translate_bedtools(builder: &mut Builder, args: Vec<String>, input: Resource, output: Resource) {
    let mut argp = Arguments::from_vec(args.into_iter().map(|s| s.into()).collect());
    match argp.subcommand().unwrap().as_deref() {
        Some("makewindows") => {
            // The input comes from the `-b` argument, which may be literally
            // `/dev/stdin`.
            let filename: String = argp.value_from_str("-b").unwrap();
            let input = if filename == "/dev/stdin" {
                input
            } else {
                builder.file(filename)
            };

            // Use an instruction to parse the BED file.
            let input = builder.load_bed(input);

            builder.instr(
                &[input],
                output,
                Op::MakeWindows {
                    size: argp.value_from_str("-w").unwrap(),
                },
            );
        }
        _ => unimplemented!("unsupported bedtools subcommand"),
    }
}

fn command_to_ir(builder: &mut Builder, command: Command, input: Resource, output: Resource) {
    let Command::Simple(simple) = command else {
        unimplemented!("only simple commands supported");
    };

    let name = word_str(simple.word_or_name.expect("command name"));

    let mut args = vec![];
    let mut redirects = vec![];
    if let Some(suffix) = simple.suffix {
        for item in suffix.0 {
            match item {
                CommandPrefixOrSuffixItem::Word(w) => args.push(word_str(w)),
                CommandPrefixOrSuffixItem::IoRedirect(r) => redirects.push(r),
                _ => unimplemented!(),
            }
        }
    }

    translate_command(builder, name, args, redirects, input, output);
}

fn pipeline_to_ir(builder: &mut Builder, pipeline: Pipeline, input: Resource, output: Resource) {
    // Step through the pipeline and construct a pipe between each consecutive
    // pair.
    let mut input = input;
    let last = pipeline.seq.len() - 1;
    for (i, step) in pipeline.seq.into_iter().enumerate() {
        let output = if i == last {
            output
        } else {
            builder.rsrc(ir::ResourceKind::Pipe)
        };
        command_to_ir(builder, step, input, output);
        input = output; // Feed this pipe into the next step.
    }
}

fn item_to_ir(builder: &mut Builder, item: CompoundListItem, input: Resource, output: Resource) {
    if let SeparatorOperator::Async = item.1 {
        unimplemented!("async commands not supported");
    }

    if !item.0.additional.is_empty() {
        unimplemented!("&& and || not supported");
    }

    pipeline_to_ir(builder, item.0.first, input, output);
}

pub fn sh_to_ir(shell: Program) -> ir::Program {
    let mut builder = Builder::new();
    for list in shell.complete_commands {
        for item in list.0 {
            item_to_ir(&mut builder, item, Resource::stdin(), Resource::stdout());
        }
    }
    builder.build()
}

/// Convert a `brush_parser` "word" atom into a plain string.
///
/// For example, both `"foo bar"` and `foo\ bar` become `foo bar`.
fn word_str(word: Word) -> String {
    let opts = brush_parser::ParserOptions::default();
    let mut buf = String::new();
    let pieces = brush_parser::word::parse(&word.value, &opts).unwrap();
    flatten_word_pieces(&mut buf, pieces);
    buf
}

fn flatten_word_pieces(buf: &mut String, pieces: Vec<WordPieceWithSource>) {
    for piece in pieces {
        match piece.piece {
            WordPiece::Text(s) => buf.push_str(&s),
            WordPiece::SingleQuotedText(s) => buf.push_str(&s),
            WordPiece::EscapeSequence(s) => {
                // We expect this to be a backslash and then a single character.
                assert_eq!(s.len(), 2);
                let mut chars = s.chars();
                assert_eq!(chars.next().unwrap(), '\\');
                buf.push(chars.next().unwrap());
            }
            WordPiece::DoubleQuotedSequence(ps) => flatten_word_pieces(buf, ps),
            _ => unimplemented!(),
        }
    }
}
