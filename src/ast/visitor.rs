use super::*;

pub trait Visitor {
    fn visit_crate(&mut self, krate: &Crate);
    fn visit_stmt(&mut self, stmt: &Stmt);
    fn visit_expr(&mut self, expr: &Expr);
    fn visit_ident(&mut self, ident: &Ident);
}

impl dyn Visitor {
    fn go(&mut self, krate: &Crate) {
        self.walk_crate(krate);
    }

    fn walk_crate(&mut self, krate: &Crate) {
        self.visit_crate(krate);
        for stmt in &krate.stmts {
            {
                self.walk_stmt(stmt);
            }
        }
    }

    fn walk_stmt(&mut self, stmt: &Stmt) {
        self.visit_stmt(stmt);
        match &stmt.kind {
            StmtKind::ExprStmt(expr) => self.walk_expr(expr),
            StmtKind::Let(ident) => self.walk_ident(ident),
        }
    }

    fn walk_ident(&mut self, ident: &Ident) {
        self.visit_ident(ident);
    }

    fn walk_expr(&mut self, expr: &Expr) {
        self.visit_expr(expr);
        match &expr.kind {
            ExprKind::NumLit(_) => (),
            ExprKind::Binary(_op, l, r) => {
                self.walk_expr(l);
                self.walk_expr(r);
            }
            ExprKind::Unary(_op, inner) => {
                self.walk_expr(inner);
            }
        }
    }
}
