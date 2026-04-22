mod eval;
mod ir;
mod parse;
mod pretty;

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

fn run_line(line: &str, pretend: bool) {
    let shell = parse::parse_sh(line);
    let prog = parse::sh_to_ir(shell);
    if pretend {
        print!("{}", prog);
    } else {
        eval::run(prog);
    }
}

fn repl(pretend: bool) -> rustyline::Result<()> {
    let mut rl = DefaultEditor::new()?;
    loop {
        match rl.readline("$ ") {
            Ok(line) => run_line(&line, pretend),
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
    let mut args = pico_args::Arguments::from_env();
    let pretend = args.contains(["-p", "--pretend"]);
    repl(pretend).unwrap();
}
