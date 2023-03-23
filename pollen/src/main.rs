extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::Parser;
use ast::*;

#[derive(Parser)]
#[grammar = "pollen.pest"]
pub struct PollenParser;

lazy_static::lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {
        use pest::pratt_parser::{Assoc::*, Op};
        use Rule::*;

        // Precedence is defined lowest to highest
        PrattParser::new()
            .op(Op::infix(or, left))
            .op(Op::infix(and, left))
            .op(Op::infix(eq, left) | Op::infix(neq, left))
            .op(Op::infix(gt, left) | Op::infix(lt, left) 
                | Op::infix(geq, left) | Op::infix(leq, left))
            // Addition and subtract have equal precedence
            .op(Op::infix(add, Left) | Op::infix(sub, Left))
            .op(Op::infix(mul, Left) | Op::infix(div, left) | Op::infix(modulo, Left))
    };
}

pub fn parse_expr(pairs: Pairs<Rule>) -> Expr {
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::integer_lit => Expr::Integer(primary.as_str().parse::<i32>().unwrap()),
            Rule::true_lit => Expr::Bool(true),
            Rule::false_lift => Expr::Bool(false),
            Rule::char_lit => Expr::Char(primary.as_str()[0] as char)
            Rule::string_lit => Expr::StringLit(primary.as_str().to_string())
            Rule::identifier => Expr::Id(primary.as_str().to_string())
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule)
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::add => BinOp::Add,
                Rule::sub => BinOp::Sub,
                Rule::mult => BinOp::Mult,
                Rule::div => BinOp::Div,
                Rule::modulo => BinOp::Mod,
                Rule::exp => BinOp::Exp,
                Rule::lt => BinOp::Lt,
                Rule::gt => BinOp::Gt,
                Rule::leq => BinOp::Leq,
                Rule::geq => BinOp::Geq,
                Rule::eq => BinOp::Eq,
                Rule::neq => BinOp::Neq,
                Rule::and => BinOp::And,
                Rule::or => BinOp::Or
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            Expr::BinOp {
                lhs: Box::new(parse_expr(lhs)),
                op,
                rhs: Box::new(parse_expr(rhs)),
            }
        })
        .parse(pairs)

}

pub fn main() {
    println!("grammar is valid");
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn bool1() {
        assert_eq!(
            format!("{:?}", PollenParser::parse(Rule::true, "true")),
            "Ok([Pair { rule: bool_lit, span: Span { str: \"true\", start: 0, end: 4 }, inner: [] }])"
        )
    }

    #[test]
    fn bool2() {
        assert_eq!(
            format!("{:?}", PollenParser::parse(Rule::false, "false")),
            "Ok([Pair { rule: bool_lit, span: Span { str: \"false\", start: 0, end: 5 }, inner: [] }])"
        )
    }
    
    // TODO: Make these tests more concise using ? keyword
    #[test]
    fn char_lit1() {
        assert_eq!(
            format!("{:?}", PollenParser::parse(Rule::char_lit, "'c'")),
            "Ok([Pair { rule: char_lit, span: Span { str: \"'c'\", start: 0, end: 3 }, inner: [] }])"
        )
    }

    #[test]
    fn char_lit2() {
        assert_eq!(
            format!("{:?}", PollenParser::parse(Rule::char_lit, "'Z'")),
            "Ok([Pair { rule: char_lit, span: Span { str: \"'Z'\", start: 0, end: 3 }, inner: [] }])"
        )
    }

    // TODO: Test backspace characters once I can parse a file
    #[test]
    #[ignore]
    fn char_lit3() {
        assert_eq!(
            format!("{:?}", PollenParser::parse(Rule::char_lit, "'\\''")),
            "Ok([Pair { rule: char_lit, span: Span { str: \"'\\''\", start: 0, end: 5 }, inner: [] }])"
        )
    }

    #[test]
    #[ignore]
    fn char_lit4() {
        assert_eq!(
            format!("{:?}", PollenParser::parse(Rule::char_lit, "'\\'")),
            "Ok([Pair { rule: char_lit, span: Span { str: \"'\\\\'\", start: 0, end: 4 }, inner: [] }])"
        )
    }

    #[test]
    #[ignore] // TODO: Test this when file parsing cababilities are added
    fn string_lit1() {
        assert_eq!(
            format!("{:?}", PollenParser::parse(Rule::string_lit, "\"string\"")),
            "Ok([Pair { rule: string_lit, span: Span { str: \"\"string\"\", start: 0, end: 8 }, inner: [] }])"
        )
    }

    #[test]
    #[ignore] // TODO: Test this when file parsing cababilities are added
    fn string_lit2() {
        assert_eq!(
            format!("{:?}", PollenParser::parse(Rule::string_lit, "\"'\"")),
            "Ok([Pair { rule: string_lit, span: Span { str: \"\"'\"\", start: 0, end: 3 }, inner: [] }])"
        )
    }

    #[test]
    fn id1() {
        assert_eq!(
            format!("{:?}", PollenParser::parse(Rule::identifier, "Var1")),
            "Ok([Pair { rule: identifier, span: Span { str: \"Var1\", start: 0, end: 4 }, inner: [] }])"
        )
    }

    #[test]
    fn id2() {
        assert_eq!(
            format!("{:?}", PollenParser::parse(Rule::identifier, "1v")),
            "Err(Error { variant: ParsingError { positives: [identifier], negatives: [] }, location: Pos(0), line_col: Pos((1, 1)), path: None, line: \"1v\", continued_line: None })"
        )
    }
}