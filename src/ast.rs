use crate::{file::FilePosition, token::TokenType};

pub struct Decl {
    pub pos: FilePosition,
    pub kind: DeclKind
}

pub enum DeclKind {
    Function {
        name: String,
        params: Vec<String>,
        body: Stmt
    },

    Data {
        name: String,
        count: i32,
        initial: Option<Expr>
    }
}

pub struct Stmt {
    pub pos: FilePosition,
    pub kind: StmtKind
}

pub enum StmtKind {
    Block(Vec<Stmt>),
    Expr(Expr),
    Auto(String, i32, Option<Expr>),
    Extern(String),
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),
    While(Expr, Box<Stmt>),
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
