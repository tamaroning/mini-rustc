use super::{Codegen, LLValue};
use crate::{
    ast::{Block, Crate, Func, ItemKind, LetStmt, Stmt, StmtKind},
    backend_llvm::{LLImm, LLReg, LLTy},
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
        let Some(body) = &func.body else{
            return Ok(());
        };

        let frame = self.compute_frame(func);
        self.push_frame(frame);

        println!(
            "define {} @{}() {{",
            self.ty_to_llty(&func.ret_ty).to_string(),
            func.name.symbol
        );

        let body_val = self.gen_block(body)?;

        if !self.ctx.get_type(body.id).is_never() {
            println!("\tret {}", body_val.to_string_with_type());
        }

        println!("}}");

        self.pop_frame();

        Ok(())
    }

    pub fn gen_block(&mut self, block: &'a Block) -> Result<LLValue, ()> {
        let mut last_stmt_val = None;
        for stmt in &block.stmts {
            last_stmt_val = Some(self.gen_stmt(stmt)?);
        }
        let ret = last_stmt_val.unwrap_or(LLValue::Imm(LLImm::Void));
        Ok(ret)
    }

    fn gen_stmt(&mut self, stmt: &'a Stmt) -> Result<LLValue, ()> {
        println!("; Starts stmt `{}`", stmt.span.to_snippet());
        let val = match &stmt.kind {
            StmtKind::Semi(expr) => {
                self.gen_expr(expr)?;
                LLValue::Imm(LLImm::Void)
            }
            StmtKind::Expr(expr) => self.gen_expr(expr)?,
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
                LLValue::Imm(LLImm::Void)
            }
        };
        println!("; Finished stmt `{}`", stmt.span.to_snippet());
        Ok(val)
    }
}
