use std::os::macos::raw::stat;

use flash::parser::{Node, Redirect};

fn parse(input: &str) -> Node {
    // Following the example from the flash README.
    use flash::lexer::Lexer;
    use flash::parser::Parser;
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    parser.parse_script()
}

#[derive(Debug)]
enum GraphResource {
    File(String),
}

#[derive(Debug)]
struct DepthOp {
    input: GraphResource,
    path: Option<String>,
}

#[derive(Debug)]
enum Op {
    Depth(DepthOp),
}

#[derive(Debug)]
struct Program {
    ops: Vec<Op>,
}

fn cmd_to_ir(name: String, args: Vec<String>, redirects: Vec<Redirect>) -> Op {
    if name == "odgi" {
        if args[0] == "depth" {
            dbg!(args);
            dbg!(redirects);
            Op::Depth(DepthOp {
                input: GraphResource::File("hi".to_string()),
                path: None,
            })
        } else {
            unimplemented!("unsupported odgi subcommand");
        }
    } else {
        unimplemented!("only odgi commands are supported");
    }
}

fn script_to_ir(shell: Node) -> Program {
    match shell {
        Node::List {
            statements,
            operators,
        } => {
            let ops: Vec<_> = statements
                .into_iter()
                .map(|statement| match statement {
                    Node::Command {
                        name,
                        args,
                        redirects,
                    } => cmd_to_ir(name, args, redirects),
                    _ => unimplemented!(),
                })
                .collect();
            Program { ops }
        }
        _ => unimplemented!(),
    }
}

fn main() {
    let shell = parse("odgi depth -i chr8.pan.og -r 'chm13#chr8'");
    dbg!(&shell);
    dbg!(&script_to_ir(shell));
}
