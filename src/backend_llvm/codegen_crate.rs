use super::{Codegen, LLValue};
use crate::{
    ast::{Block, Crate, Func, ItemKind, LetStmt, Stmt, StmtKind},
    backend_llvm::{frame::LocalKind, LLImm},
    resolve::BindingKind,
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

        print!(
            "define {} @{}(",
            self.ty_to_llty(&func.ret_ty).to_string(),
            func.name.symbol
        );

        // arguments
        let mut it = self
            .peek_frame()
            .get_locals()
            .iter()
            .filter(|(binding, _)| binding.kind == BindingKind::Arg)
            .peekable();
        while let Some((_, local)) = it.next() {
            print!("{}", local.reg.to_string_with_type());
            if it.peek().is_some() {
                print!(", ");
            }
        }

        println!(") {{");

        for (name_binding, local) in self.peek_frame().get_locals() {
            if name_binding.kind == BindingKind::Let {
                assert!(local.kind == LocalKind::Ptr);
                println!(
                    "\t{} = alloca {}",
                    local.reg.name,
                    local.reg.llty.peel_ptr().unwrap().to_string()
                );
            }
        }

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
            StmtKind::Let(LetStmt { ident, ty: _, init }) => {
                if let Some(init) = init {
                    let ident_reg = self.gen_ident_lval(ident).unwrap();
                    let init_val = self.gen_expr(init)?;
                    // TODO: initializer
                    println!(
                        "\tstore {}, {}",
                        init_val.to_string_with_type(),
                        ident_reg.to_string_with_type()
                    );
                }
                LLValue::Imm(LLImm::Void)
            }
        };
        println!("; Finished stmt `{}`", stmt.span.to_snippet());
        Ok(val)
    }
}
