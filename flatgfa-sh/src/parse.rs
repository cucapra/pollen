use crate::ir::{self, Builder};
use flash::parser::{Node, Redirect};
use pico_args::Arguments;

pub fn parse_sh(input: &str) -> Node {
    // Following the example from the flash README.
    use flash::lexer::Lexer;
    use flash::parser::Parser;
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    parser.parse_script()
}

fn cmd_to_ir(builder: &mut Builder, name: String, args: Vec<String>, _redirects: Vec<Redirect>) {
    if name == "odgi" {
        let mut argp = Arguments::from_vec(args.into_iter().map(|s| s.into()).collect());
        match argp.subcommand().unwrap().as_deref() {
            Some("depth") => {
                let input = match argp.opt_value_from_str("-i").unwrap() {
                    Some(filename) => builder.add_file(filename),
                    None => builder.stdin(),
                };
                let op = ir::Op::Depth(ir::DepthOp {
                    input,
                    output: builder.stdout(),
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
