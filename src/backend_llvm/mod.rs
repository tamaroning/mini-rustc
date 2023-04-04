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
    next_reg: usize,
}

impl<'a> Codegen<'a> {
    fn new(ctx: &'a mut Ctxt) -> Self {
        Codegen { ctx, next_reg: 1 }
    }

    fn get_fresh_reg(&mut self) -> String {
        let i = self.next_reg;
        self.next_reg += 1;
        format!("%{i}")
    }

    fn ty_to_llty(&self, ty: &Ty) -> LLTy {
        match ty {
            Ty::Unit => LLTy::Void,
            Ty::I32 => LLTy::I32,
            _ => panic!(),
        }
    }

    fn go(&mut self, krate: &'a Crate) -> Result<(), ()> {
        println!(r#"target triple = "x86_64-unknown-linux-gnu""#);
        println!("");
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

        println!(
            "define {} @{}() {{",
            self.ty_to_llty(&func.ret_ty).to_string(),
            func.name.symbol
        );
        if let Some(body) = &func.body {
            for stmt in &body.stmts {
                self.codegen_stmt(stmt)?;
            }
        }
        //println!("\tret void");
        println!("}}");
        Ok(())
    }

    fn codegen_stmt(&mut self, stmt: &'a Stmt) -> Result<(), ()> {
        println!("; Starts stmt `{}`", stmt.span.to_snippet());
        let store_kind = match &stmt.kind {
            StmtKind::Semi(expr) => {
                self.codegen_expr(expr)?;
            }
            StmtKind::Expr(expr) => {
                self.codegen_expr(expr)?;
            }
            StmtKind::Let(_) => {
                // TODO:
            }
        };
        println!("; Finished stmt `{}`", stmt.span.to_snippet());
        Ok(store_kind)
    }

    fn codegen_expr(&mut self, expr: &'a Expr) -> Result<Option<RegAndLLTy>, ()> {
        println!("; Starts expr `{}`", expr.span.to_snippet());
        let ret = match &expr.kind {
            ExprKind::NumLit(n) => {
                let reg = self.get_fresh_reg();
                let llty = LLTy::I32;
                println!("\t{reg} = bitcast i32 {} to i32", n);
                Ok(Some(RegAndLLTy::new(reg, llty)))
            }
            ExprKind::Return(inner) => {
                let RegAndLLTy { reg, llty } = self.codegen_expr(inner)?.unwrap();
                println!("\tret {} {}", llty.to_string(), reg);
                Ok(None)
            }
            _ => panic!(),
        };
        println!("; Starts expr `{}`", expr.span.to_snippet());
        ret
    }
}

enum LLTy {
    Void,
    I32,
}

impl LLTy {
    fn to_string(&self) -> String {
        match self {
            LLTy::Void => "void".to_string(),
            LLTy::I32 => "i32".to_string(),
        }
    }
}

struct RegAndLLTy {
    reg: String,
    llty: LLTy,
}

impl RegAndLLTy {
    fn new(reg: String, llty: LLTy) -> Self {
        RegAndLLTy { reg, llty }
    }
}
