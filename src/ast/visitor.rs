use super::*;

pub trait Visitor<'ctx> {
    fn visit_crate(&mut self, krate: &'ctx Crate);
    fn visit_stmt(&mut self, stmt: &'ctx Stmt);
    fn visit_let_stmt(&mut self, let_stmt: &'ctx LetStmt);
    fn visit_expr(&mut self, expr: &'ctx Expr);
    fn visit_ident(&mut self, ident: &'ctx Ident);
}

pub fn go<'ctx>(v: &mut dyn Visitor<'ctx>, krate: &'ctx Crate) {
    walk_crate(v, krate);
}

fn walk_crate<'ctx>(v: &mut dyn Visitor<'ctx>, krate: &'ctx Crate) {
    v.visit_crate(krate);
    for stmt in &krate.stmts {
        {
            walk_stmt(v, stmt);
        }
    }
}

fn walk_stmt<'ctx>(v: &mut dyn Visitor<'ctx>, stmt: &'ctx Stmt) {
    v.visit_stmt(stmt);
    match &stmt.kind {
        StmtKind::ExprStmt(expr) => walk_expr(v, expr),
        StmtKind::Let(let_stmt) => walk_let_stmt(v, let_stmt),
    }
}

fn walk_let_stmt<'ctx>(v: &mut dyn Visitor<'ctx>, let_stmt: &'ctx LetStmt) {
    v.visit_let_stmt(let_stmt);
    let LetStmt { ident } = let_stmt;
    walk_ident(v, ident);
}

fn walk_ident<'ctx>(v: &mut dyn Visitor<'ctx>, ident: &'ctx Ident) {
    v.visit_ident(ident);
}

fn walk_expr<'ctx>(v: &mut dyn Visitor<'ctx>, expr: &'ctx Expr) {
    v.visit_expr(expr);
    match &expr.kind {
        ExprKind::NumLit(_) => (),
        ExprKind::Binary(_op, l, r) => {
            walk_expr(v, l);
            walk_expr(v, r);
        }
        ExprKind::Unary(_op, inner) => {
            walk_expr(v, inner);
        }
    }
}
