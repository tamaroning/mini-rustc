use crate::ty::Ty;
use std::rc::Rc;

pub mod visitor;

pub type NodeId = u32;

#[derive(Debug)]
pub struct Crate {
    pub items: Vec<Func>,
}

#[derive(Debug)]
pub struct Func {
    pub name: Ident,
    pub body: Block,
    pub id: NodeId,
}

#[derive(Debug)]
pub struct Stmt {
    pub kind: StmtKind,
}

#[derive(Debug)]
pub enum StmtKind {
    /// Expression without trailing semicolon
    Expr(Box<Expr>),
    /// Expression with trailing semicolon
    Semi(Box<Expr>),
    Let(LetStmt),
}

#[derive(Debug)]
pub struct LetStmt {
    pub ident: Ident,
    pub ty: Rc<Ty>,
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
    BoolLit(bool),
    Ident(Ident),
    Assign(Box<Expr>, Box<Expr>),
    Return(Box<Expr>),
    Call(Ident),
    Block(Block),
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Eq,
    Ne,
    Gt,
    Lt,
}

#[derive(Debug)]
pub enum UnOp {
    Plus,
    Minus,
}
