use crate::ir::{self, Builder, Op, Resource};
use brush_parser::ast::{
    Command, CommandPrefixOrSuffixItem, CompoundListItem, IoFileRedirectKind, IoFileRedirectTarget,
    IoRedirect, Pipeline, Program, SeparatorOperator,
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

fn cmd_to_ir(
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
                    IoFileRedirectTarget::Filename(w) => w.value,
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
        let mut argp = Arguments::from_vec(args.into_iter().map(|s| s.into()).collect());
        match argp.subcommand().unwrap().as_deref() {
            Some("depth") => {
                // Possibly override input from `-i` flag.
                if let Some(filename) = argp.opt_value_from_str(["-i", "--input"]).unwrap() {
                    input = builder.file(filename);
                }

                // Get a FlatGFA data structure, either by parsing GFA text or
                // by memory-mapping a FlatGFA binary file.
                let input = builder.load_gfa(input);

                // In the `odgi depth` command line, the default is a per-path
                // table. `-d` switches to a per-node table, and `-b` takes a
                // BED file and switches to an interval table.
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
    } else if name == "bedtools" {
        let mut argp = Arguments::from_vec(args.into_iter().map(|s| s.into()).collect());
        match argp.subcommand().unwrap().as_deref() {
            Some("makewindows") => {
                // The input comes from the `-b` argument, which may be
                // literally `/dev/stdin`.
                let filename: String = argp.value_from_str("-b").unwrap();
                if filename != "/dev/stdin" {
                    input = builder.file(filename);
                }

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
    } else {
        // Any non-odgi command is a "passthrough" shell command.
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

fn command_to_ir(builder: &mut Builder, command: Command, input: Resource, output: Resource) {
    let Command::Simple(simple) = command else {
        unimplemented!("only simple commands supported");
    };

    let name = simple.word_or_name.expect("command name").value;
    let mut args = vec![];
    let mut redirects = vec![];
    if let Some(suffix) = simple.suffix {
        for item in suffix.0 {
            match item {
                CommandPrefixOrSuffixItem::Word(w) => args.push(w.value),
                CommandPrefixOrSuffixItem::IoRedirect(r) => redirects.push(r),
                _ => unimplemented!(),
            }
        }
    }

    cmd_to_ir(builder, name, args, redirects, input, output);
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
    let mut builder = ir::Builder::new();
    let stdin = builder.stdin();
    let stdout = builder.stdout();
    for list in shell.complete_commands {
        for item in list.0 {
            item_to_ir(&mut builder, item, stdin, stdout);
        }
    }
    builder.build()
}
