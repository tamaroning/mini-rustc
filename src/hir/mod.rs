use crate::{
    resolve::CanonicalPath,
    span::{Ident, Span},
};

pub mod visitor;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HirId {
    private: u32,
}

impl HirId {
    pub fn new() -> HirId {
        HirId { private: 0 }
    }

    pub fn next(&self) -> HirId {
        HirId {
            private: self.private + 1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LocalDefId {
    private: u32,
}

impl LocalDefId {
    pub fn new() -> LocalDefId {
        LocalDefId { private: 0 }
    }

    pub fn next(&self) -> LocalDefId {
        LocalDefId {
            private: self.private + 1,
        }
    }

    pub fn dummy() -> LocalDefId {
        LocalDefId { private: u32::MAX }
    }
}

#[derive(Debug)]
pub struct Mod<'hir> {
    pub items: Vec<&'hir Item<'hir>>,
    pub id: HirId,
}

#[derive(Debug)]
pub struct Item<'hir> {
    pub kind: ItemKind<'hir>,
}

#[derive(Debug)]
pub enum ItemKind<'hir> {
    Func(FuncDef),
    Struct(StructDef),
    Mod(Mod<'hir>),
}

#[derive(Debug)]
pub struct ForeignMod {
    pub funcs: Vec<FuncDef>,
}

#[derive(Debug)]
pub struct StructDef {
    pub fields: Vec<(Ident, Ty)>,
}

#[derive(Debug)]
pub struct FuncDef {
    pub params: Vec<(Ident, Ty)>,
    pub ret_ty: Ty,
    /// Extern abi
    pub ext: Option<String>,
    pub body: Option<Block>,
}

#[derive(Debug)]
pub struct Stmt {
    pub kind: StmtKind,
    pub id: HirId,
    pub span: Span,
}

#[derive(Debug)]
pub enum StmtKind {
    /// Expression without trailing semicolon
    Expr(Box<Expr>),
    /// Expression with trailing semicolon
    Semi(Box<Expr>),
    Let(Let),
}

#[derive(Debug)]
pub struct Let {
    pub ident: Ident,
    pub ty: Option<Ty>,
    pub init: Option<Expr>,
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub id: HirId,
    pub span: Span,
}

#[derive(Debug)]
pub enum ExprKind {
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    NumLit(u32),
    BoolLit(bool),
    StrLit(String),
    Unit,
    Path(Ident),
    Assign(Box<Expr>, Box<Expr>),
    Return(Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Block(Block),
    /// cond, then (only block expr), else
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    Index(Box<Expr>, Box<Expr>),
    Field(Box<Expr>, Ident),
    Struct(Ident, Vec<(Ident, Box<Expr>)>),
    Array(Vec<Expr>),
}

#[derive(Debug)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
    pub id: HirId,
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

#[derive(Debug)]
pub struct Ty {
    pub kind: TyKind,
    pub span: Span,
}

#[derive(Debug)]
pub enum TyKind {
    Unit,
    Bool,
    I32,
    Str,
    Array(Box<Ty>, usize),
    Adt(CanonicalPath),
    Ref(Option<Region>, Box<Ty>),
    Never,
}

pub type Region = String;
