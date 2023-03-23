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

/* 
#[derive(Debug)]
pub enum UOp {
    Neg,
    Not
}
*/

#[derive(Debug)]
pub enum Expr {
    Integer(i32),
    Bool(bool),
    Char(char),
    StringLit(String),
    Id(String),
    BinOpExpr{
        lhs: Box<Expr>,
        op: BinOp,
        rhs: Box<Expr>
     } /*,
    UOpExpr{
        op: UOp,
        expr: Box<Expr
    }
    */
}