mod eval;
mod ir;
mod parse;
mod pretty;

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

fn run_line(line: &str) {
    let shell = parse::parse_sh(&line);
    let prog = parse::sh_to_ir(shell);
    print!("{}", prog);
    eval::run(prog);
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
