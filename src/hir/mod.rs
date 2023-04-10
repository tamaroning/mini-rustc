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
}

#[derive(Debug)]
pub struct Crate<'hir> {
    pub items: Vec<Item<'hir>>,
    pub id: HirId,
}

#[derive(Debug)]
pub struct Item<'hir> {
    pub kind: ItemKind<'hir>,
}

#[derive(Debug)]
pub enum ItemKind<'hir> {
    Func(&'hir FuncDef<'hir>),
    Struct(&'hir StructDef<'hir>),
    Mod(Mod<'hir>),
}

#[derive(Debug)]
pub struct Mod<'hir> {
    pub items: Vec<&'hir Item<'hir>>,
    pub id: HirId,
}

#[derive(Debug)]
pub struct ForeignMod<'hir> {
    pub funcs: Vec<&'hir FuncDef<'hir>>,
}

#[derive(Debug)]
pub struct StructDef<'hir> {
    pub fields: Vec<(Ident, &'hir Ty<'hir>)>,
}

#[derive(Debug)]
pub struct FuncDef<'hir> {
    pub params: Vec<(Ident, &'hir Ty<'hir>)>,
    pub ret_ty: &'hir Ty<'hir>,
    /// Extern abi
    pub ext: Option<String>,
    pub body: Option<Block<'hir>>,
}

#[derive(Debug)]
pub struct Stmt<'hir> {
    pub kind: StmtKind<'hir>,
    pub id: HirId,
    pub span: Span,
}

#[derive(Debug)]
pub enum StmtKind<'hir> {
    /// Expression without trailing semicolon
    Expr(Box<Expr<'hir>>),
    /// Expression with trailing semicolon
    Semi(Box<Expr<'hir>>),
    Let(Let<'hir>),
}

#[derive(Debug)]
pub struct Let<'hir> {
    pub ident: Ident,
    pub ty: Option<&'hir Ty<'hir>>,
    pub init: Option<Expr<'hir>>,
}

#[derive(Debug)]
pub struct Expr<'hir> {
    pub kind: ExprKind<'hir>,
    pub id: HirId,
    pub span: Span,
}

#[derive(Debug)]
pub enum ExprKind<'hir> {
    Binary(BinOp, Box<Expr<'hir>>, Box<Expr<'hir>>),
    Unary(UnOp, Box<Expr<'hir>>),
    NumLit(u32),
    BoolLit(bool),
    StrLit(String),
    Unit,
    Path(Ident),
    Assign(Box<Expr<'hir>>, Box<Expr<'hir>>),
    Return(Box<Expr<'hir>>),
    Call(Box<Expr<'hir>>, Vec<Expr<'hir>>),
    Block(Block<'hir>),
    /// cond, then (only block expr), else
    If(Box<Expr<'hir>>, Box<Expr<'hir>>, Option<Box<Expr<'hir>>>),
    Index(Box<Expr<'hir>>, Box<Expr<'hir>>),
    Field(Box<Expr<'hir>>, Ident),
    Struct(Ident, Vec<(Ident, Box<Expr<'hir>>)>),
    Array(Vec<Expr<'hir>>),
}

#[derive(Debug)]
pub struct Block<'hir> {
    pub stmts: Vec<Stmt<'hir>>,
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
pub struct Ty<'hir> {
    pub kind: TyKind<'hir>,
    pub span: Span,
}

#[derive(Debug)]
pub enum TyKind<'hir> {
    Unit,
    Bool,
    I32,
    Str,
    Array(&'hir Ty<'hir>, usize),
    Adt(CanonicalPath),
    Ref(Option<Region>, &'hir Ty<'hir>),
    Never,
}

pub type Region = String;
