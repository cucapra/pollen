use crate::ir;
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

pub fn cmd_to_ir(name: String, args: Vec<String>, redirects: Vec<Redirect>) -> ir::Op {
    if name == "odgi" {
        let mut argp = Arguments::from_vec(args.into_iter().map(|s| s.into()).collect());
        match argp.subcommand().unwrap().as_deref() {
            Some("depth") => ir::Op::Depth(ir::DepthOp {
                input: ir::GraphResource::File(argp.value_from_str("-i").unwrap()),
                path: argp.opt_value_from_str("-r").unwrap(),
            }),
            _ => unimplemented!("unsupported odgi subcommand"),
        }
    } else {
        unimplemented!("only odgi commands are supported");
    }
}

pub fn script_to_ir(shell: Node) -> ir::Program {
    let mut builder = ir::Builder::new();

    match shell {
        Node::List {
            statements,
            operators,
        } => {
            for statement in statements {
                match statement {
                    Node::Command {
                        name,
                        args,
                        redirects,
                    } => builder.add_op(cmd_to_ir(name, args, redirects)),
                    _ => unimplemented!(),
                }
            }
        }
        _ => unimplemented!(),
    }

    builder.build()
}
