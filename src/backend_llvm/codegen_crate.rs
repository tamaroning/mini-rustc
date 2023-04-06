use super::Codegen;
use crate::{
    ast::{Block, Crate, Func, ItemKind, LetStmt, Stmt, StmtKind},
    backend_llvm::{LLReg, LLTy},
};

impl<'a> Codegen<'a> {
    pub fn gen_crate(&mut self, krate: &'a Crate) -> Result<(), ()> {
        for item in &krate.items {
            match &item.kind {
                ItemKind::Func(func) => {
                    self.gen_func(func)?;
                }
                ItemKind::Struct(_) => (),
                ItemKind::ExternBlock(_) => (),
            }
        }
        Ok(())
    }

    fn gen_func(&mut self, func: &'a Func) -> Result<(), ()> {
        // do not generate code for the func if it does not have its body
        if func.body.is_none() {
            return Ok(());
        }

        let frame = self.compute_frame(func);
        self.push_frame(frame);

        println!(
            "define {} @{}() {{",
            self.ty_to_llty(&func.ret_ty).to_string(),
            func.name.symbol
        );
        let body_res = self.gen_block(func.body.as_ref().unwrap())?;

        match body_res {
            Some(LLReg { name, llty }) => println!("\tret {} {}", llty.to_string(), name),
            None => {
                if self.ty_to_llty(&func.ret_ty) == LLTy::Void {
                    println!("\tret void");
                }
            }
        }
        println!("}}");

        self.pop_frame();

        Ok(())
    }

    pub fn gen_block(&mut self, block: &'a Block) -> Result<Option<LLReg>, ()> {
        let mut last_stmt_res = None;
        for stmt in &block.stmts {
            last_stmt_res = self.gen_stmt(stmt)?;
        }
        Ok(last_stmt_res)
    }

    fn gen_stmt(&mut self, stmt: &'a Stmt) -> Result<Option<LLReg>, ()> {
        println!("; Starts stmt `{}`", stmt.span.to_snippet());
        let res = match &stmt.kind {
            StmtKind::Semi(expr) => {
                self.gen_expr(expr)?;
                Ok(None)
            }
            StmtKind::Expr(expr) => Ok(self.gen_expr(expr)?),
            StmtKind::Let(LetStmt {
                ident,
                ty: _,
                init: _,
            }) => {
                // if let stmt is in a loop, memory might be allocated inifinitely
                let name = self.ctx.resolver.resolve_ident(ident).unwrap();
                let reg = self.peek_frame().get_local(&name);
                println!(
                    "\t{} = alloca {}",
                    reg.name,
                    reg.llty.peel_ptr().to_string()
                );
                // TODO: initializer
                Ok(None)
            }
        };
        println!("; Finished stmt `{}`", stmt.span.to_snippet());
        res
    }
}
