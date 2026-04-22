use crate::ir::{self, Builder};
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

fn cmd_to_ir(builder: &mut Builder, name: String, args: Vec<String>, redirects: Vec<Redirect>) {
    // Do the input or output come from stream redirections?
    let mut input = builder.stdin();
    let mut output = builder.stdout();
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
                match argp.opt_value_from_str(["-i", "--input"]).unwrap() {
                    Some(filename) => input = builder.file(filename),
                    None => (),
                };

                let op = ir::Op::Depth(ir::DepthOp {
                    input,
                    output,
                    path: argp.opt_value_from_str("-r").unwrap(),
                });
                builder.add_op(op);
            }
            _ => unimplemented!("unsupported odgi subcommand"),
        }
    } else {
        unimplemented!("only odgi commands are supported");
    }
}

fn node_to_ir(builder: &mut Builder, node: Node) {
    match node {
        Node::List {
            statements,
            operators: _,
        } => {
            for statement in statements {
                match statement {
                    Node::Command {
                        name,
                        args,
                        redirects,
                    } => cmd_to_ir(builder, name, args, redirects),
                    Node::Comment(_) => (),
                    _ => unimplemented!(),
                }
            }
        }
        _ => unimplemented!(),
    }
}

pub fn sh_to_ir(shell: Node) -> ir::Program {
    let mut builder = ir::Builder::new();
    node_to_ir(&mut builder, shell);
    builder.build()
}
