use flash::parser::Node;

fn parse(input: &str) -> Node {
    // Following the example from the flash README.
    use flash::lexer::Lexer;
    use flash::parser::Parser;
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    parser.parse_script()
}

fn main() {
    let line = parse("odgi depth -i chr8.pan.og -r chm13#chr8");
    dbg!(line);
}
