use crate::ir::{self, Builder, Op, Resource};
use flash::parser::{Node, Redirect, RedirectKind};
use pico_args::Arguments;

pub fn parse_sh(input: &str) -> Node {
    // Following the example from the flash README.
    use flash::lexer::Lexer;
    use flash::parser::Parser;
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    parser.parse_script()
}

fn cmd_to_ir(
    builder: &mut Builder,
    name: String,
    args: Vec<String>,
    redirects: Vec<Redirect>,
    input: Resource,
    output: Resource,
) {
    // Do the input or output come from stream redirections?
    let mut input = input;
    let mut output = output;
    for redirect in redirects {
        match redirect.kind {
            RedirectKind::Input => input = builder.file(redirect.file),
            RedirectKind::Output => output = builder.file(redirect.file),
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
                // table, and `-d` switches to a per-node table. (There are
                // other modes, such as `-D`, to support...)
                if argp.contains("-d") {
                    builder.instr(input, output, Op::NodeDepth);
                } else {
                    builder.instr(
                        input,
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
                    input,
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
            input,
            output,
            Op::Exec {
                command: name,
                args,
            },
        );
    }
}

fn node_to_ir(builder: &mut Builder, node: Node, input: Resource, output: Resource) {
    match node {
        Node::Command {
            name,
            args,
            redirects,
        } => cmd_to_ir(builder, name, args, redirects, input, output),
        Node::Comment(_) => (),
        Node::Pipeline { commands } => {
            // Step through the pipeline and construct a pipe
            // between each consecutive pair.
            let mut input = input;
            let last = commands.len() - 1;
            for (i, step) in commands.into_iter().enumerate() {
                let output = if i == last {
                    output
                } else {
                    builder.rsrc(ir::ResourceKind::Pipe)
                };
                node_to_ir(builder, step, input, output);
                input = output; // Feed this pipe into the next step.
            }
        }
        Node::List {
            statements,
            operators: _,
        } => {
            for statement in statements {
                node_to_ir(builder, statement, input, output);
            }
        }
        _ => unimplemented!(),
    }
}

pub fn sh_to_ir(shell: Node) -> ir::Program {
    let mut builder = ir::Builder::new();
    let stdin = builder.stdin();
    let stdout = builder.stdout();
    node_to_ir(&mut builder, shell, stdin, stdout);
    builder.build()
}
