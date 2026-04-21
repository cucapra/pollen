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

impl Program {
    fn run(&self) {
        for op in &self.ops {
            op.run();
        }
    }
}

impl Op {
    fn run(&self) {
        match self {
            Self::Depth(op) => op.run(),
        }
    }
}

impl DepthOp {
    fn run(&self) {
        println!(
            "here I would run depth with input {:?} and optional path name {:?}",
            self.input, self.path
        );
    }
}

fn cmd_to_ir(name: String, args: Vec<String>, redirects: Vec<Redirect>) -> Op {
    if name == "odgi" {
        let mut argp = Arguments::from_vec(args.into_iter().map(|s| s.into()).collect());
        match argp.subcommand().unwrap().as_deref() {
            Some("depth") => Op::Depth(DepthOp {
                input: GraphResource::File(argp.value_from_str("-i").unwrap()),
                path: argp.opt_value_from_str("-r").unwrap(),
            }),
            _ => unimplemented!("unsupported odgi subcommand"),
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
