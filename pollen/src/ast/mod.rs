#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mult,
    Div,
    Mod,
    Exp,
    Lt,
    Gt,
    Leq,
    Geq,
    Eq,
    Neq,
    And,
    Or
}

#[derive(Debug)]
pub enum UOp {
    Not 
}

#[derive(Debug)]
pub struct Id(pub String);

#[derive(Debug)]
pub enum Expr {
    Integer(i32),
    Bool(bool),
    Char(char),
    StringLit(String),
    Var(Id),
    BinOpExpr{
        lhs: Box<Expr>,
        op: BinOp,
        rhs: Box<Expr>
    },
    UOpExpr{
        op: UOp,
        expr: Box<Expr>
    },
    FieldAccess{
        object: Box<Expr>,
        field: Box<Expr>
    }
}

#[derive(Debug)]
pub enum Typ {
    Int,
    Bool,
    Char,
    Node,
    Step,
    Edge,
    Base,
    String,
    Strand,
    Tuple(Box<Typ>, Box<Typ>)
}

#[derive(Debug)]
pub enum Stmt {
    Decl {
        typ: Typ,
        id: Id,
        expr: Option<Expr>,
    },
    Assign {
        id: Id,
        expr: Expr
    },
    Block {
        stmts: Vec<Box<Stmt>>
    }
}

#[derive(Debug)]
pub struct Prog {
    pub stmts: Vec<Stmt>,
}