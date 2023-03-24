pub struct Expr {
    pub kind: ExprKind,
}

pub enum ExprKind {
    Binary(BinOp, Box<Expr>, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    NumLit(u32),
}

pub enum BinOp {
    Add,
    Sub,
    Mul,
}

pub enum UnOp {
    Plus,
    Minus,
}
