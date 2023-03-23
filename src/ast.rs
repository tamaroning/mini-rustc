pub struct Expr {
    pub kind: ExprKind,
}

pub enum ExprKind {
    NumLit(u32),
}
