use crate::ir::{self, Builder, ResourceRef};
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
    input: ResourceRef,
    output: ResourceRef,
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

                // In the `odgi depth` command line, the default is a per-path
                // table, and `-d` switches to a per-node table. (There are
                // other modes, such as `-D`, to support...)
                if argp.contains("-d") {
                    builder.add_instr(ir::NodeDepthInstr { input, output });
                } else {
                    builder.add_instr(ir::PathDepthInstr {
                        input,
                        output,
                        path: argp.opt_value_from_str("-r").unwrap(),
                    });
                };
            }
            _ => unimplemented!("unsupported odgi subcommand"),
        }
    } else {
        // Any non-odgi command is a "passthrough" shell command.
        builder.add_instr(ir::ExecInstr {
            input,
            output,
            command: name,
            args,
        });
    }
}

fn node_to_ir(builder: &mut Builder, node: Node, input: ResourceRef, output: ResourceRef) {
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
                let output = if i == last { output } else { builder.pipe() };
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
