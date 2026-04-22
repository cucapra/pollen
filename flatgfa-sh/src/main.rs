mod ir;

use flash::parser::{Node, Redirect};
use pico_args::Arguments;
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

fn parse(input: &str) -> Node {
    // Following the example from the flash README.
    use flash::lexer::Lexer;
    use flash::parser::Parser;
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    parser.parse_script()
}

fn cmd_to_ir(name: String, args: Vec<String>, redirects: Vec<Redirect>) -> ir::Op {
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

fn script_to_ir(shell: Node) -> ir::Program {
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
            ir::Program { ops }
        }
        _ => unimplemented!(),
    }
}

fn run_line(line: &str) {
    let shell = parse(&line);
    let prog = script_to_ir(shell);
    prog.run();
}

fn repl() -> rustyline::Result<()> {
    let mut rl = DefaultEditor::new()?;
    loop {
        match rl.readline("$ ") {
            Ok(line) => run_line(&line),
            Err(ReadlineError::Interrupted) => break,
            Err(ReadlineError::Eof) => break,
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}

fn main() {
    repl().unwrap();
}
