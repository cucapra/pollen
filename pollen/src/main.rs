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
            .op(Op::infix(Rule::field_access, Left))
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
        Rule::graph_decl => {
            let mut inner = stmt.into_inner();
            let id = {
                let Some(pair) = inner.next() else {
                    unreachable!("A graph declaration requires an Id")
                };
                parse_id(pair)
            };
            Stmt::GraphDecl {
                id: id
            }
        },
        Rule::parset_decl => {
            let mut inner = stmt.into_inner();
            let id = {
                let Some(pair) = inner.next() else {
                    unreachable!("A parset declaration requires an Id")
                };
                parse_id(pair)
            };
            let typ = {
                let Some(pair) = inner.next() else {
                    unreachable!("A parset declaration requires a type")
                };
                parse_typ(pair)
            };
            let graph_id = {
                if let Some(pair) = inner.next() {
                    Some(parse_id(pair))
                } else {
                    None
                }
            };
            Stmt::ParsetDecl {
                id: id,
                typ: typ,
                graph_id: graph_id
            }
        },
        Rule::assign => {
            // Just contains an id and an expression
            let mut inner = stmt.into_inner();
            let id = {
                let Some(pair) = inner.next() else {
                    unreachable!("A declaration requires an Id")
                };
                parse_id(pair)
            };
            let expr = {
                parse_expr(inner)
            };
            Stmt::Assign {
                id: id,
                expr: expr
            }
        },
        Rule::block => {
            let mut stmts = Vec::new();
            for s in stmt.into_inner() {
                stmts.push(Box::new(parse_stmt(s)));
            }
            Stmt::Block { stmts: stmts }
        },
        Rule::if_stmt => {
            let mut inner = stmt.into_inner();
            // if guard
            let guard = {
                let Some(pair) = inner.next() else {
                    unreachable!("An if statement requires a guard")
                };
                parse_expr(pair.into_inner())
            };
            // if block
            let if_block = {
                let Some(pair) = inner.next() else {
                    unreachable!("No if block found")
                };
                Box::new(parse_stmt(pair))
            };
            let mut else_block = None;
            struct ElifStmt{
                guard: Expr,
                block: Stmt
            }
            let mut elif_stmts = Vec::new();
            while let Some(pair) = inner.next() {
                match pair.as_rule() {
                    // else block
                    Rule::block => {
                        else_block = {
                            Some(Box::new(parse_stmt(pair)))
                        };
                    },
                    _ => {
                        // elifs consume the next two pairs
                        let elif_guard = {
                            parse_expr(pair.into_inner())
                        };
                        let elif_block = {
                            let Some(pair) = inner.next() else {
                                unreachable!("No elif block found")
                            };
                            parse_stmt(pair)
                        };
                        elif_stmts.push(
                            ElifStmt{ guard: elif_guard, block: elif_block}
                        );
                    }
                }
            }
            // TODO: Questionable cloning happening here
            let elif = elif_stmts.iter().rfold(None, 
                |next_elif, elif_stmt| {
                    let ElifStmt { guard, block } = elif_stmt;
                    Some(Box::new(
                        Stmt::If {
                            guard: guard.clone(),
                            if_block: Box::new(block.clone()),
                            elif_block: next_elif,
                            else_block: None
                        }
                    ))
                }
            );

            Stmt::If {
                guard : guard,
                if_block : if_block,
                elif_block: elif,
                else_block: else_block,
            }
        },
        Rule::while_stmt => {
            // Contains a guard and a block
            let mut inner = stmt.into_inner();
            let guard = {
                let Some(pair) = inner.next() else {
                    unreachable!("While loop has no guard")
                };
                parse_expr(pair.into_inner())
            };
            let block = {
                let Some(pair) = inner.next() else {
                    unreachable!("While loop has no body")
                };
                parse_stmt(pair)
            };
            Stmt::While {
                guard: guard,
                body: Box::new(block)
            }
        },
        Rule::for_stmt => {
            // Contains a guard and a block
            let mut inner = stmt.into_inner();
            let id = {
                let Some(pair) = inner.next() else {
                    unreachable!("For loop with no id")
                };
                parse_id(pair)
            };
            let iterator = {
                let Some(pair) = inner.next() else {
                    unreachable!("For loop with no iterator")
                };
                parse_expr(pair.into_inner())
            };
            let body = {
                let Some(pair) = inner.next() else {
                    unreachable!("For loop with no body")
                };
                parse_stmt(pair)
            };
            Stmt::For {
                id: id,
                iterator: iterator,
                body: Box::new(body)
            }
        },
        Rule::emit_to => {
            // Contains an expression and a set identifier
            let mut inner = stmt.into_inner();
            let expr = {
                let pair = inner.next().unwrap();
                parse_expr(pair.into_inner())
            };
            let set_id = {
                let pair = inner.next().unwrap();
                parse_id(pair)
            };
            Stmt::EmitTo {
                expr: expr,
                set_id: set_id
            }
        },
        Rule::call_stmt => {
            let object = parse_expr(stmt.into_inner());
            match object {
                Expr::FuncCall{ name, args } => {
                    Stmt::FuncCallStmt {
                        name: name,
                        args: args 
                    }
                },
                Expr::MethodCall{ object, method, args } => {
                    Stmt::MethodCallStmt {
                        object: *object,
                        method: method,
                        args: args
                    }
                },
                _ => { panic!("A call_stmt must either be a function call or a method call") }
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

fn parse_call_args(arg_list: Pair<Rule>) -> Vec<Expr> {
    match arg_list.as_rule() {
        Rule::call_args => {
            let mut args = Vec::new();
            for arg in arg_list.into_inner() {
                args.push(
                    parse_expr(arg.into_inner())
                );
            }
        args
        },
        rule => {panic!("{:?} is not a list of function arguments", rule)}
    }
}

fn parse_expr(expression: Pairs<Rule>) -> Expr {
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
            Rule::record_lit => {
                // record_lit looks like Record { f1: e1, ..., fn:en }
                let mut inner = primary.into_inner();
                let typ = {
                    let Some(pair) = inner.next() else {
                        unreachable!("An if statement requires a guard")
                    };
                    parse_typ(pair)
                };
                let mut fields = Vec::new();
                while let Some(pair) = inner.next() {
                    // Consume the next two pairs
                    let field = {
                        parse_id(pair)
                    };
                    let val = {
                        let Some(pair) = inner.next() else {
                            unreachable!("Each field needs a value")
                        };
                        parse_expr(pair.into_inner())
                    };
                    fields.push(
                        RecordField{ field: field, val: val}
                    );
                }
                Expr::Record {
                    typ: typ,
                    fields: fields
                }
            },
            Rule::record_update_lit => {
                // record_update_lit looks like { r1 with f1: e1, ..., fn:en }
                let mut inner = primary.into_inner();
                let parent = {
                    let Some(pair) = inner.next() else {
                        unreachable!("An if statement requires a guard")
                    };
                    parse_id(pair)
                };
                let mut fields = Vec::new();
                while let Some(pair) = inner.next() {
                    // Consume the next two pairs
                    let field = {
                        parse_id(pair)
                    };
                    let val = {
                        let Some(pair) = inner.next() else {
                            unreachable!("Each field needs a value")
                        };
                        parse_expr(pair.into_inner())
                    };
                    fields.push(
                        RecordField{ field: field, val: val}
                    );
                }
                Expr::RecordUpdate {
                    parent: parent,
                    fields: fields
                }
            },
            Rule::obj_initialize => {
                let typ = parse_typ(primary.into_inner().next().unwrap());
                Expr::ObjInitialization{ typ: typ }
            },
            Rule::func_call => {
                let mut inner = primary.into_inner();
                let func_name = {
                    let Some(pair) = inner.next() else {
                        unreachable!("A function call requires a name")
                    };
                    parse_id(pair)
                };
                let args = {
                    let Some(pair) = inner.next() else {
                        unreachable!("A function call needs 0 or more arguments")
                    };
                    parse_call_args(pair)
                };
                Expr::FuncCall {
                    name: func_name,
                    args: args
                }
            },
            Rule::expr => {
                // If this rule has been reached then 
                // this is a parenthesized expression

                // println!("Full expr: {:?}", inner);
                parse_expr(primary.into_inner())
            }
            rule => unreachable!("Expr::parse expected atom, found {:?}", rule)
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
        .map_postfix(|lhs, op| {
            match op.as_rule() {
                rule => unreachable!("{:?} not recognized as a postfix", rule),
            }
        })
        .map_infix(|lhs, op, rhs| {
            enum OpType {
                Binary(BinOp),
                FieldAccess
            }
            let op_typ = match op.as_rule() {
                Rule::add => OpType::Binary(BinOp::Add),
                Rule::sub => OpType::Binary(BinOp::Sub),
                Rule::mult => OpType::Binary(BinOp::Mult),
                Rule::div => OpType::Binary(BinOp::Div),
                Rule::modulo => OpType::Binary(BinOp::Mod),
                Rule::lt => OpType::Binary(BinOp::Lt),
                Rule::gt => OpType::Binary(BinOp::Gt),
                Rule::leq => OpType::Binary(BinOp::Leq),
                Rule::geq => OpType::Binary(BinOp::Geq),
                Rule::eq => OpType::Binary(BinOp::Eq),
                Rule::neq => OpType::Binary(BinOp::Neq),
                Rule::and => OpType::Binary(BinOp::And),
                Rule::or => OpType::Binary(BinOp::Or),
                Rule::field_access => OpType::FieldAccess,
                rule => unreachable!("Expr::parse expected infix operation, found {:?}", rule),
            };
            match op_typ {
                OpType::Binary(op) => {
                    Expr::BinOpExpr {
                        lhs: Box::new(lhs),
                        op,
                        rhs: Box::new(rhs),
                    }
                }
                OpType::FieldAccess => {
                    match rhs {
                        Expr::FuncCall { name, args } => {
                            Expr::MethodCall {
                                object: Box::new(lhs),
                                method: name,
                                args: args
                            }
                        },
                        _ => {
                            Expr::FieldAccess {
                                object: Box::new(lhs),
                                field: Box::new(rhs)
                            }
                       }
                    }
                }
            }
        })
        .parse(expression)
}

fn parse_id(id: Pair<Rule>) -> Id {
    match id.as_rule() {
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
            "Segment" => Typ::Node,
            "Step" => Typ::Step,
            "Edge" => Typ::Edge,
            "Base" => Typ::Base,
            "String" => Typ::String,
            "Strand" => Typ::Strand,
            t => unreachable!("Unknown type: {}", t)
        },
        // Rule::tuple_typ => {
        //     let mut inner = typ.into_inner();
        //     let t1 = {
        //         if let Some(pair) = inner.next() {
        //             parse_typ(pair)
        //         } else {
        //             unreachable!("Expected first tuple type but found nothing")
        //         }
        //     };
        //     let t2 = {
        //         if let Some(pair) = inner.next() {
        //             parse_typ(pair)
        //         } else {
        //             unreachable!("Expected second tuple type but found nothing")
        //         }
        //     };
        //     assert!(inner.next().is_none());
        //     Typ::Tuple(Box::new(t1), Box::new(t2))
        // },
        Rule::set_typ => {
            let mut inner = typ.into_inner();
            let t = {
                if let Some(pair) = inner.next() {
                    parse_typ(pair)
                } else {
                    unreachable!("Expected first tuple type but found nothing")
                }
            };
            assert!(inner.next().is_none());
            Typ::Set(Box::new(t))
        }
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

    let mut prog: String = match args.len() {
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

    match PollenParser::parse(Rule::file, &prog) {
        Ok(mut pairs) => {
            // println!(
            //     "Pre-parsed: {:#?}",
            //     pairs
            // );
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