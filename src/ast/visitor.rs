use super::*;

/// AST visitor
pub trait Visitor<'ctx>: Sized {
    fn visit_crate(&mut self, _krate: &'ctx Crate) {}
    fn visit_crate_post(&mut self, _krate: &'ctx Crate) {}
    fn visit_func(&mut self, _func: &'ctx Func) {}
    fn visit_func_post(&mut self, _func: &'ctx Func) {}
    fn visit_struct_item(&mut self, _struct: &'ctx StructItem) {}
    fn visit_struct_item_post(&mut self, _struct: &'ctx StructItem) {}
    fn visit_stmt(&mut self, _stmt: &'ctx Stmt) {}
    fn visit_stmt_post(&mut self, _stmt: &'ctx Stmt) {}
    fn visit_let_stmt(&mut self, _let_stmt: &'ctx LetStmt) {}
    fn visit_let_stmt_post(&mut self, _let_stmt: &'ctx LetStmt) {}
    fn visit_expr(&mut self, _expr: &'ctx Expr) {}
    fn visit_expr_post(&mut self, _expr: &'ctx Expr) {}
    fn visit_block(&mut self, _block: &'ctx Block) {}
    fn visit_block_post(&mut self, _block: &'ctx Block) {}
    fn visit_ident(&mut self, _ident: &'ctx Ident) {}
    fn visit_type(&mut self, _ty: &'ctx Ty) {}
}

pub fn go<'ctx, V: Visitor<'ctx>>(v: &mut V, krate: &'ctx Crate) {
    walk_crate(v, krate);
}

pub fn go_func<'ctx, V: Visitor<'ctx>>(v: &mut V, func: &'ctx Func) {
    walk_func(v, func);
}

fn walk_crate<'ctx, V: Visitor<'ctx>>(v: &mut V, krate: &'ctx Crate) {
    v.visit_crate(krate);
    for item in &krate.items {
        {
            match &item.kind {
                ItemKind::Func(func) => {
                    walk_func(v, func);
                }
                ItemKind::Struct(struct_item) => {
                    walk_struct_item(v, struct_item);
                }
            }
        }
    }
    v.visit_crate_post(krate);
}

fn walk_func<'ctx, V: Visitor<'ctx>>(v: &mut V, func: &'ctx Func) {
    v.visit_func(func);
    walk_ident(v, &func.name);
    for stmt in &func.body.stmts {
        {
            walk_stmt(v, stmt);
        }
    }
    v.visit_func_post(func);
}

fn walk_struct_item<'ctx, V: Visitor<'ctx>>(v: &mut V, struct_item: &'ctx StructItem) {
    v.visit_struct_item(struct_item);
    for (ident, ty) in &struct_item.fields {
        {
            walk_ident(v, ident);
            walk_type(v, ty);
        }
    }
    v.visit_struct_item_post(struct_item);
}

fn walk_stmt<'ctx, V: Visitor<'ctx>>(v: &mut V, stmt: &'ctx Stmt) {
    v.visit_stmt(stmt);
    match &stmt.kind {
        StmtKind::Semi(expr) => walk_expr(v, expr),
        StmtKind::Expr(expr) => walk_expr(v, expr),
        StmtKind::Let(let_stmt) => walk_let_stmt(v, let_stmt),
    }
    v.visit_stmt_post(stmt);
}

fn walk_let_stmt<'ctx, V: Visitor<'ctx>>(v: &mut V, let_stmt: &'ctx LetStmt) {
    v.visit_let_stmt(let_stmt);
    let LetStmt { ident, ty } = let_stmt;
    walk_ident(v, ident);
    walk_type(v, ty);
    v.visit_let_stmt(let_stmt);
}

fn walk_ident<'ctx, V: Visitor<'ctx>>(v: &mut V, ident: &'ctx Ident) {
    v.visit_ident(ident);
}

fn walk_type<'ctx, V: Visitor<'ctx>>(v: &mut V, ty: &'ctx Ty) {
    v.visit_type(ty);
}

fn walk_expr<'ctx, V: Visitor<'ctx>>(v: &mut V, expr: &'ctx Expr) {
    v.visit_expr(expr);
    match &expr.kind {
        ExprKind::NumLit(_) | ExprKind::BoolLit(_) => (),
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
        ExprKind::Call(func, args) => {
            walk_expr(v, func);
            for arg in args {
                walk_expr(v, arg);
            }
        }
        ExprKind::Block(block) => {
            walk_block(v, block);
        }
        ExprKind::If(cond, then, els) => {
            walk_expr(v, cond);
            walk_expr(v, then);
            if let Some(els) = els {
                walk_expr(v, els);
            }
        }
        ExprKind::Index(array, index) => {
            walk_expr(v, array);
            walk_expr(v, index);
        }
        ExprKind::Field(receiver, field) => {
            walk_expr(v, receiver);
            walk_ident(v, field);
        }
    }
    v.visit_expr_post(expr);
}

fn walk_block<'ctx, V: Visitor<'ctx>>(v: &mut V, block: &'ctx Block) {
    v.visit_block(block);
    for stmt in &block.stmts {
        walk_stmt(v, stmt);
    }
    v.visit_block_post(block);
}
