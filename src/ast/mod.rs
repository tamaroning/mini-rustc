use std::rc::Rc;

use crate::ty::Ty;

pub mod visitor;

pub type NodeId = u32;

#[derive(Debug)]
pub struct Crate {
    pub items: Vec<Func>,
}

#[derive(Debug)]
pub struct Func {
    pub name: Ident,
    pub stmts: Vec<Stmt>,
    pub id: NodeId,
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
