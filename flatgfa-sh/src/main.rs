mod eval;
mod ir;
mod parse;
mod pretty;

use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;

fn run_shell(line: &str, pretend: bool) {
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
            Ok(line) => run_shell(&line, pretend),
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
    let cmd: Option<String> = args.opt_value_from_str("-c").unwrap();
    let script_filename: Option<String> = args.opt_free_from_str().unwrap();

    if let Some(cmd) = cmd {
        run_shell(&cmd, pretend);
    } else if let Some(filename) = script_filename {
        let script = std::fs::read_to_string(filename).unwrap();
        run_shell(&script, pretend);
    } else {
        repl(pretend).unwrap();
    }
}

#[cfg(test)]
mod tests {
    /// Run some CLI tests from the README.
    #[test]
    fn readme_tests() {
        trycmd::TestCases::new().case("README.md");
    }
}
