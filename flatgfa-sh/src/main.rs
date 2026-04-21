use yash_syntax::syntax;

fn parse(input: &str) -> syntax::List {
    // Following the example from the yash_syntax docs:
    // https://docs.rs/yash-syntax/latest/yash_syntax/parser/index.html
    use futures_executor::block_on;
    use yash_syntax::{
        input::Memory,
        parser::{Parser, lex::Lexer},
    };

    let input = Box::new(Memory::new(input));
    let mut lexer = Lexer::new(input);
    let mut parser = Parser::new(&mut lexer);
    block_on(parser.command_line()).unwrap().unwrap()
}

fn main() {
    let line = parse("foo | bar --baz > qux");
    dbg!(line);
}
