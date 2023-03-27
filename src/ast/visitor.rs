use super::*;

/// AST visitor
pub trait Visitor<'ctx>: Sized {
    fn visit_crate(&mut self, _krate: &'ctx Crate) {}
    fn visit_crate_post(&mut self, _krate: &'ctx Crate) {}
    fn visit_stmt(&mut self, _stmt: &'ctx Stmt) {}
    fn visit_stmt_post(&mut self, _stmt: &'ctx Stmt) {}
    fn visit_let_stmt(&mut self, _let_stmt: &'ctx LetStmt) {}
    fn visit_let_stmt_post(&mut self, _let_stmt: &'ctx LetStmt) {}
    fn visit_expr(&mut self, _expr: &'ctx Expr) {}
    fn visit_expr_post(&mut self, _expr: &'ctx Expr) {}
    fn visit_ident(&mut self, _ident: &'ctx Ident) {}
    fn visit_ident_post(&mut self, _ident: &'ctx Ident) {}
}

pub fn go<'ctx, V: Visitor<'ctx>>(v: &mut V, krate: &'ctx Crate) {
    walk_crate(v, krate);
}

fn walk_crate<'ctx, V: Visitor<'ctx>>(v: &mut V, krate: &'ctx Crate) {
    v.visit_crate(krate);
    for stmt in &krate.stmts {
        {
            walk_stmt(v, stmt);
        }
    }
    v.visit_crate_post(krate);
}

fn walk_stmt<'ctx, V: Visitor<'ctx>>(v: &mut V, stmt: &'ctx Stmt) {
    v.visit_stmt(stmt);
    match &stmt.kind {
        StmtKind::ExprStmt(expr) => walk_expr(v, expr),
        StmtKind::Let(let_stmt) => walk_let_stmt(v, let_stmt),
    }
    v.visit_stmt_post(stmt);
}

fn walk_let_stmt<'ctx, V: Visitor<'ctx>>(v: &mut V, let_stmt: &'ctx LetStmt) {
    v.visit_let_stmt(let_stmt);
    let LetStmt { ident } = let_stmt;
    walk_ident(v, ident);
    v.visit_let_stmt(let_stmt);
}

fn walk_ident<'ctx, V: Visitor<'ctx>>(v: &mut V, ident: &'ctx Ident) {
    v.visit_ident(ident);
    v.visit_ident_post(ident);
}

fn walk_expr<'ctx, V: Visitor<'ctx>>(v: &mut V, expr: &'ctx Expr) {
    v.visit_expr(expr);
    match &expr.kind {
        ExprKind::NumLit(_) => (),
        ExprKind::Binary(_, l, r) | ExprKind::Assign(l, r) => {
            walk_expr(v, l);
            walk_expr(v, r);
        }
        ExprKind::Unary(_op, inner) => {
            walk_expr(v, inner);
        }
        ExprKind::Ident(ident) => {
            walk_ident(v, ident);
        }
        ExprKind::Return(inner) => {
            walk_expr(v, inner);
        }
    }
    v.visit_expr_post(expr);
}
