use crate::{file::FilePosition, token::TokenType};

pub struct Decl {
    pub pos: FilePosition,
    pub kind: DeclKind
}

pub struct Variable(pub String, pub Option<Expr>);

pub enum DeclKind {
    Function {
        name: String,
        params: Vec<String>,
        body: Stmt
    },

    External(Variable)
}

pub struct Stmt {
    pub pos: FilePosition,
    pub kind: StmtKind
}

pub enum StmtKind {
    Block(Vec<Stmt>),
    Expr(Expr),
    Auto(Variable),
    Extern(String),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Option<Box<Stmt>>),
    DoWhile(Expr, Box<Stmt>),
    Return(Option<Expr>),
    Break,
    Continue
}

pub struct Expr {
    pub pos: FilePosition,
    pub kind: ExprKind
}

pub enum ExprKind {
    IntLit(i32),
    StringLit(String),
    Var(String),
    UnaryOp(TokenType, bool, Box<Expr>),
    BinOp(Box<Expr>, TokenType, Box<Expr>),
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>)
}
