use crate::ast::{self, Crate, Expr, ExprKind, Func, ItemKind, LetStmt, Stmt, StmtKind};
use crate::middle::ty::Ty;
use crate::middle::Ctxt;

pub fn compile(ctx: &mut Ctxt, krate: &Crate) -> Result<(), ()> {
    codegen(ctx, krate)?;

    Ok(())
}

pub fn codegen(ctx: &mut Ctxt, krate: &Crate) -> Result<(), ()> {
    let mut codegen = Codegen::new(ctx);
    codegen.go(krate)?;
    Ok(())
}

struct Codegen<'a> {
    ctx: &'a mut Ctxt,
}

impl<'a> Codegen<'a> {
    fn new(ctx: &'a mut Ctxt) -> Self {
        Codegen { ctx }
    }

    fn ty_to_llty(&self, ty: &Ty) -> String {
        match ty {
            Ty::Unit => "void".to_string(),
            _ => panic!(),
        }
    }

    fn go(&mut self, krate: &'a Crate) -> Result<(), ()> {
        self.codegen_crate(krate)?;
        // TODO: literals
        Ok(())
    }

    fn codegen_crate(&mut self, krate: &'a Crate) -> Result<(), ()> {
        for item in &krate.items {
            match &item.kind {
                ItemKind::Func(func) => {
                    self.codegen_func(func)?;
                }
                ItemKind::Struct(_) => (),
                ItemKind::ExternBlock(_) => (),
            }
        }
        Ok(())
    }

    fn codegen_func(&mut self, func: &'a Func) -> Result<(), ()> {
        // do not generate code for the func if it does not have its body
        if func.body.is_none() {
            return Ok(());
        }

        print!(
            "define internal {}@{}() {{",
            self.ty_to_llty(&func.ret_ty),
            func.name.symbol
        );
        if let Some(body) = &func.body {
            for stmt in &body.stmts {
                self.codegen_stmt(stmt)?;
            }
        }
        println!("\tret void");
        println!("}}");
        Ok(())
    }

    fn codegen_stmt(&mut self, stmt: &'a Stmt) -> Result<(), ()> {
        println!("# Starts stmt `{}`", stmt.span.to_snippet());
        let store_kind = match &stmt.kind {
            StmtKind::Semi(expr) => {
                self.codegen_expr(expr)?;
            }
            StmtKind::Expr(expr) => self.codegen_expr(expr)?,
            StmtKind::Let(LetStmt { ident, ty, init }) => {
                // TODO:
            }
        };
        println!("# Finished stmt `{}`", stmt.span.to_snippet());
        Ok(store_kind)
    }

    fn codegen_expr(&mut self, expr: &'a Expr) -> Result<(), ()> {
        println!("; Starts expr `{}`", expr.span.to_snippet());
        match &expr.kind {
            ExprKind::NumLit(n) => {
                println!("\tmov rax, {}", n);
            }
            _ => panic!(),
        }
        Ok(())
    }
}
