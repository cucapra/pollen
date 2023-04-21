#[derive(Debug)]
#[derive(Clone)]
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
    Tuple(Box<Typ>, Box<Typ>),
    Set(Box<Typ>)
}

#[derive(Debug)]
#[derive(Clone)]
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
#[derive(Clone)]
pub enum UOp {
    Not 
}

#[derive(Debug)]
#[derive(Clone)]
pub struct Id(pub String);

#[derive(Debug)]
#[derive(Clone)]
pub struct RecordField{
    pub field: Id,
    pub val: Expr
}

#[derive(Debug)]
#[derive(Clone)]
pub enum Expr {
    Integer(i32),
    Bool(bool),
    Char(char),
    StringLit(String),
    Var(Id),
    BinOpExpr {
        lhs: Box<Expr>,
        op: BinOp,
        rhs: Box<Expr>
    },
    UOpExpr {
        op: UOp,
        expr: Box<Expr>
    },
    Record {
        typ: Typ,
        fields: Vec<RecordField>
    },
    RecordUpdate {
        parent: Id,
        fields: Vec<RecordField>
    },
    FieldAccess {
        object: Box<Expr>,
        field: Box<Expr>
    }
}

#[derive(Debug)]
#[derive(Clone)]
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
    },
    If {
        guard: Expr,
        if_block: Box<Stmt>, // Block stmt
        elif_block: Option<Box<Stmt>>, // If stmt
        else_block: Option<Box<Stmt>> // Block stmt
    },
    While {
        guard: Expr,
        body: Box<Stmt>
    },
    For {
        id: Id,
        iterator: Expr,
        body: Box<Stmt>
    },
    EmitTo {
        expr: Expr,
        set_id: Id
    }
}

#[derive(Debug)]
pub struct Prog {
    pub stmts: Vec<Stmt>,
}