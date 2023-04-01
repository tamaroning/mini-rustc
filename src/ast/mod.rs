use crate::ty::Ty;
use std::rc::Rc;

pub mod visitor;

pub type NodeId = u32;

#[derive(Debug)]
pub struct Crate {
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub struct Item {
    pub kind: ItemKind,
}

#[derive(Debug)]
pub enum ItemKind {
    Func(Func),
    Struct(StructItem),
    ExternBlock(ExternBlock),
}

#[derive(Debug)]
pub struct ExternBlock {
    pub funcs: Vec<Func>,
}

#[derive(Debug)]
pub struct StructItem {
    pub ident: Ident,
    pub fields: Vec<(Ident, Rc<Ty>)>,
}

#[derive(Debug)]
pub struct Func {
    pub name: Ident,
    pub params: Vec<(Ident, Rc<Ty>)>,
    pub ret_ty: Rc<Ty>,
    /// Extern abi
    pub ext: Option<String>,
    pub body: Option<Block>,
    pub id: NodeId,
}

#[derive(Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub id: NodeId,
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
    pub init: Option<Expr>,
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
    StrLit(String),
    Unit,
    Ident(Ident),
    Assign(Box<Expr>, Box<Expr>),
    Return(Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Block(Block),
    /// cond, then (only block expr), else
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    Index(Box<Expr>, Box<Expr>),
    Field(Box<Expr>, Ident),
    Struct(Ident, Vec<(Ident, Box<Expr>)>),
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
