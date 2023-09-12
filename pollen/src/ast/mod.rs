#[derive(Debug)]
#[derive(Clone)]
pub struct Id(pub String);

#[derive(Debug)]
pub enum Import {
    Import { file: String },
    ImportAs { file: String, name: Id },
    ImportFrom { file: String, funcs: Vec<(Id, Option<Id>)> }
}

#[derive(Debug)]
#[derive(Clone)]
pub enum Typ {
    Int,
    Bool,
    Char,
    Path,
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
    Tuple {
        lhs : Box<Expr>,
        rhs : Box<Expr>
    },
    FieldAccess {
        object: Box<Expr>,
        field: Box<Expr>
    },
    FuncCall {
        name: Id,
        args: Vec<Expr>
    },
    MethodCall {
        object: Box<Expr>,
        method: Id,
        args: Vec<Expr>
    },
    ObjInitialization{
        typ: Typ
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
    GraphDecl {
        id: Id
    },
    ParsetDecl {
        id: Id,
        typ: Typ,
        graph_id: Option<Id>
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
    FuncCallStmt {
        name: Id,
        args: Vec<Expr>
    },
    MethodCallStmt {
        object: Expr,
        method: Id,
        args: Vec<Expr>
    },
    EmitTo {
        expr: Expr,
        set_id: Id
    }
}

#[derive(Debug)]
pub struct FuncDef {
    pub name: Id,
    pub args: Vec<(Id, Typ)>,
    pub ret_typ: Option<Typ>,
    pub stmts: Vec<Stmt>,
    pub ret: Option<Expr>
}

#[derive(Debug)]
pub struct Prog {
    pub imports: Vec<Import>,
    pub func_defs: Vec<FuncDef>,
}