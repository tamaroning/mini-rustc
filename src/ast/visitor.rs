use super::*;

/// AST visitor
pub trait Visitor<'ctx>: Sized {
    fn visit_crate(&mut self, _krate: &'ctx Crate) {}
    fn visit_crate_post(&mut self, _krate: &'ctx Crate) {}
    fn visit_item(&mut self, _item: &'ctx Item) {}
    fn visit_item_post(&mut self, _item: &'ctx Item) {}
    fn visit_module_item(&mut self, _module: &'ctx Module) {}
    fn visit_module_item_post(&mut self, _module: &'ctx Module) {}
    fn visit_func(&mut self, _func: &'ctx Func) {}
    fn visit_func_post(&mut self, _func: &'ctx Func) {}
    fn visit_struct_item(&mut self, _struct: &'ctx StructItem) {}
    fn visit_struct_item_post(&mut self, _struct: &'ctx StructItem) {}
    fn visit_extern_block(&mut self, _block: &'ctx ExternBlock) {}
    fn visit_extern_block_post(&mut self, _block: &'ctx ExternBlock) {}
    fn visit_stmt(&mut self, _stmt: &'ctx Stmt) {}
    fn visit_stmt_post(&mut self, _stmt: &'ctx Stmt) {}
    fn visit_expr(&mut self, _expr: &'ctx Expr) {}
    fn visit_expr_post(&mut self, _expr: &'ctx Expr) {}
    fn visit_block(&mut self, _block: &'ctx Block) {}
    fn visit_block_post(&mut self, _block: &'ctx Block) {}
    fn visit_type(&mut self, _ty: &'ctx Ty) {}
    fn visit_path(&mut self, _path: &'ctx Path) {}
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
        walk_item(v, item);
    }
    v.visit_crate_post(krate);
}

fn walk_item<'ctx, V: Visitor<'ctx>>(v: &mut V, item: &'ctx Item) {
    v.visit_item(item);
    match &item.kind {
        ItemKind::Func(func) => {
            walk_func(v, func);
        }
        ItemKind::Struct(struct_item) => {
            walk_struct_item(v, struct_item);
        }
        ItemKind::ExternBlock(extern_block) => {
            walk_extern_block(v, extern_block);
        }
        ItemKind::Mod(module) => {
            walk_module_item(v, module);
        }
    }
    v.visit_item_post(item);
}

fn walk_func<'ctx, V: Visitor<'ctx>>(v: &mut V, func: &'ctx Func) {
    v.visit_func(func);
    for (_param, ty) in &func.params {
        walk_type(v, ty);
    }
    if let Some(body) = &func.body {
        walk_block(v, body)
    }
    v.visit_func_post(func);
}

fn walk_struct_item<'ctx, V: Visitor<'ctx>>(v: &mut V, struct_item: &'ctx StructItem) {
    v.visit_struct_item(struct_item);
    for (_ident, ty) in &struct_item.fields {
        {
            walk_type(v, ty);
        }
    }
    v.visit_struct_item_post(struct_item);
}

fn walk_extern_block<'ctx, V: Visitor<'ctx>>(v: &mut V, block: &'ctx ExternBlock) {
    v.visit_extern_block(block);
    for func in &block.funcs {
        walk_func(v, func);
    }
    v.visit_extern_block_post(block);
}

fn walk_module_item<'ctx, V: Visitor<'ctx>>(v: &mut V, module: &'ctx Module) {
    v.visit_module_item(module);
    for item in &module.items {
        walk_item(v, item);
    }
    v.visit_module_item_post(module);
}

fn walk_stmt<'ctx, V: Visitor<'ctx>>(v: &mut V, stmt: &'ctx Stmt) {
    v.visit_stmt(stmt);
    match &stmt.kind {
        StmtKind::Semi(expr) => walk_expr(v, expr),
        StmtKind::Expr(expr) => walk_expr(v, expr),
        StmtKind::Let(let_stmt) => {
            let LetStmt { ident: _, ty, init } = let_stmt;
            if let Some(ty) = ty {
                walk_type(v, ty);
            }
            if let Some(init) = init {
                walk_expr(v, init);
            }
        }
    }
    v.visit_stmt_post(stmt);
}

fn walk_type<'ctx, V: Visitor<'ctx>>(v: &mut V, ty: &'ctx Ty) {
    v.visit_type(ty);
}

fn walk_expr<'ctx, V: Visitor<'ctx>>(v: &mut V, expr: &'ctx Expr) {
    v.visit_expr(expr);
    match &expr.kind {
        ExprKind::NumLit(_) | ExprKind::BoolLit(_) | ExprKind::StrLit(_) | ExprKind::Unit => (),
        ExprKind::Binary(_, l, r) | ExprKind::Assign(l, r) => {
            walk_expr(v, l);
            walk_expr(v, r);
        }
        ExprKind::Unary(_op, inner) => {
            walk_expr(v, inner);
        }
        ExprKind::Path(_path) => {}
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
        ExprKind::Field(receiver, _field) => {
            walk_expr(v, receiver);
        }
        ExprKind::Struct(_ident, fds) => {
            for (_ident, expr) in fds {
                walk_expr(v, expr);
            }
        }
        ExprKind::Array(elems) => {
            for e in elems {
                walk_expr(v, e);
            }
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
