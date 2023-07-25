use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[macro_use]
extern crate lazy_static;
extern crate pest;
#[macro_use]
extern crate pest_derive;

use pest::iterators::{ Pair, Pairs };
use pest::pratt_parser::{Assoc::*, Op, PrattParser};
use pest::Parser;

pub mod ast;
use crate::ast::*;

#[derive(Parser)]
#[grammar = "pollen.pest"]
pub struct PollenParser;

lazy_static! {
    static ref PRATT_PARSER: PrattParser<Rule> = {

        // Precedence is defined lowest to highest
        PrattParser::new()
            .op(Op::infix(Rule::or, Left))
            .op(Op::infix(Rule::and, Left))
            .op(Op::infix(Rule::eq, Left) | Op::infix(Rule::neq, Left))
            .op(Op::infix(Rule::gt, Left) | Op::infix(Rule::lt, Left) 
                | Op::infix(Rule::geq, Left) | Op::infix(Rule::leq, Left))
            // Addition and subtract have equal precedence
            .op(Op::infix(Rule::add, Left) | Op::infix(Rule::sub, Left))
            .op(Op::infix(Rule::mult, Left) | Op::infix(Rule::div, Left) 
                | Op::infix(Rule::modulo, Left))
            .op(Op::prefix(Rule::not))
    };
}

pub fn parse_prog(mut prog: Pairs<Rule>) -> Prog {
    let mut stmts = Vec::new();
    while let Some(stmt) = prog.next() {
        if stmt.as_rule() != Rule::EOI {
            // TODO: Edit for function defs
            stmts.push(parse_stmt(stmt)) 
        }  
    };
    Prog{ stmts: stmts }
}

fn parse_stmt(stmt: Pair<Rule>) -> Stmt {
    match stmt.as_rule() {
        Rule::decl => {
            let mut inner = stmt.into_inner();
            let id = {
                let Some(pair) = inner.next() else {
                    unreachable!("A declaration requires an Id")
                };
                parse_id(pair)
            };
            let typ = {
                let Some(pair) = inner.next() else {
                    unreachable!("Expected inner statement, found nothing")
                };
                parse_typ(pair)
            };
            let expr_opt = {
                println!("Just the expr: {:?}", inner);
                if inner.peek().is_none() {
                    // println!("This declaration does not give an initialization");
                    None 
                }
                else {
                    Some(parse_expr(inner))
                }
            };
            Stmt::Decl {
                typ: typ,
                id: id,
                expr: expr_opt
            }
        },
        Rule::stmt => {
            let mut inner = stmt.into_inner();
            let s = {
                if let Some(pair) = inner.next() {
                    parse_stmt(pair)
                } else {
                    unreachable!("Statement has no inner statement")
                }
            };
            assert!(inner.next().is_none());
            s
        }
        rule => unreachable!("{:?} Not recognized", rule)
    }
}

fn parse_expr(expression: Pairs<Rule>) -> Expr {
    println!("Expression: {:?}", expression);
    PRATT_PARSER
        .map_primary(|primary| match primary.as_rule() {
            Rule::integer_lit => ast::Expr::Integer(primary.as_str().parse::<i32>().unwrap()),
            Rule::true_lit => Expr::Bool(true),
            Rule::false_lit => Expr::Bool(false),
            Rule::char_lit => Expr::Char(parse_char(primary.into_inner().next().unwrap())),
            Rule::string_lit => {
                let mut string = String::new();
                for character in primary.into_inner() {
                    string.push(parse_char(character));
                }
                Expr::StringLit(string)
            },
            Rule::identifier => Expr::Var(parse_id(primary)),
            Rule::expr => {
                // If this rule has been reached then 
                // this is a parenthesized expression

                // println!("Full expr: {:?}", inner);
                parse_expr(primary.into_inner())
            }
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule)
        })
        .map_infix(|lhs, op, rhs| {
            let op = match op.as_rule() {
                Rule::add => BinOp::Add,
                Rule::sub => BinOp::Sub,
                Rule::mult => BinOp::Mult,
                Rule::div => BinOp::Div,
                Rule::modulo => BinOp::Mod,
                Rule::lt => BinOp::Lt,
                Rule::gt => BinOp::Gt,
                Rule::leq => BinOp::Leq,
                Rule::geq => BinOp::Geq,
                Rule::eq => BinOp::Eq,
                Rule::neq => BinOp::Neq,
                Rule::and => BinOp::And,
                Rule::or => BinOp::Or,
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            Expr::BinOpExpr {
                lhs: Box::new(lhs),
                op,
                rhs: Box::new(rhs),
            }
        })
        .map_prefix(|op, exp| {
            let op = match op.as_rule() {
                Rule::not => UOp::Not,
                rule => unreachable!("{:?} not recognized as a uop", rule),
            };
            Expr::UOpExpr {
                op,
                expr: Box::new(exp),
            }
        })
        .parse(expression)
}

fn parse_id(id: Pair<Rule>) -> Id {
    match id.as_rule(){
        Rule::identifier => Id(id.as_str().to_string()),
        rule => panic!("Identifier expected, but {:?} found", rule)
    }
}

fn parse_typ(typ: Pair<Rule>) -> Typ {
    // println!("Type Pair: {:#?}", typ);
    match typ.as_rule() {
        Rule::atomic_typ => match typ.as_str() {
            "int" => Typ::Int,
            "bool" => Typ::Bool,
            "char" => Typ::Char,
            "Node" => Typ::Node,
            "Step" => Typ::Step,
            "Edge" => Typ::Edge,
            "Base" => Typ::Base,
            "String" => Typ::String,
            "Strand" => Typ::Strand,
            t => unreachable!("Unknown type: {}", t)
        },
        Rule::tuple_typ => {
            let mut inner = typ.into_inner();
            let t1 = {
                if let Some(pair) = inner.next() {
                    parse_typ(pair)
                } else {
                    unreachable!("Expected first tuple type but found nothing")
                }
            };
            let t2 = {
                if let Some(pair) = inner.next() {
                    parse_typ(pair)
                } else {
                    unreachable!("Expected second tuple type but found nothing")
                }
            };
            assert!(inner.next().is_none());
            Typ::Tuple(Box::new(t1), Box::new(t2))
        },
        rule => unreachable!("Unknown type: {:?}", rule)
        // TODO - probably replace this with a Result<> return type
    }
}

fn parse_char(character: Pair<Rule>) -> char {
    match character.as_rule() {
        Rule::back_backslash => '\\',
        Rule::back_tab => '\t',
        Rule::back_newline => '\n',
        Rule::back_single_quote => '\'',
        Rule::back_double_quote => '\"',
        Rule::normal_char => character.as_str().chars().nth(0).unwrap(),
        rule => unreachable!("Expected char but got {:?}", rule)
    }
}

fn extract_file(filename: String) -> String {
    // Create a path to the desired file
    let path = Path::new(&filename);
    let display = path.display();

    // Open the path in read-only mode, returns `io::Result<File>`
    let mut file = match File::open(&path) {
        Err(why) => panic!("Couldn't open file {}: {}", display, why),
        Ok(file) => file,
    };

    // Read the file contents into a string, returns `io::Result<usize>`
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("Couldn't read file {}: {}", display, why),
        Ok(_) => s
    }
}

pub fn main() {
    let args: Vec<String> = env::args().collect();

    let prog: String = match args.len() {
        // one argument passed
        2 => {
            match args[1].parse() {
                Ok(filename) => extract_file(filename),
                Err(e) => panic!("Failed with error: {:?}", e)
            }
        },
        n => {
            panic!("Expected one argument but found {}", n-1);
        }
    };

    match PollenParser::parse(Rule::prog, &prog) {
        Ok(mut pairs) => {
            println!(
                "Pre-parsed: {:#?}",
                pairs
            );
            println!("Lexing");
            println!(
                "Parsed: {:#?}",
                parse_prog(pairs.next().unwrap().into_inner())
            );
        }
        Err(e) => {
            eprintln!("Parse failed: {:?}", e);
        }
    }
}