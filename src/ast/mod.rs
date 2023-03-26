pub mod visitor;

pub type NodeId = u32;

#[derive(Debug)]
pub struct Crate {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub struct Stmt {
    pub kind: StmtKind,
}

#[derive(Debug)]
pub enum StmtKind {
    ExprStmt(Box<Expr>),
    Let(LetStmt),
}

#[derive(Debug)]
pub struct LetStmt {
    pub ident: Ident,
}

#[derive(Debug)]
pub struct Ident {
    pub symbol: String,
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub id: NodeId,
}

#[derive(Debug)]
pub enum ExprKind {
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    NumLit(u32),
    Ident(Ident),
    Assign(Box<Expr>, Box<Expr>),
}

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
}

#[derive(Debug)]
pub enum UnOp {
    Plus,
    Minus,
}
