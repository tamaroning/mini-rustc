use super::{Codegen, LLValue};
use crate::{
    ast::{Block, Crate, ExternBlock, Func, ItemKind, LetStmt, Stmt, StmtKind},
    backend_llvm::{
        frame::{compute_frame, LocalKind},
        LLImm,
    },
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
                ItemKind::ExternBlock(ext_block) => self.gen_external_block(ext_block)?,
            }
        }
        Ok(())
    }

    pub fn gen_external_block(&mut self, ext_block: &'a ExternBlock) -> Result<(), ()> {
        for func in &ext_block.funcs {
            self.gen_func(func)?;
        }
        Ok(())
    }

    fn gen_func(&mut self, func: &'a Func) -> Result<(), ()> {
        // do not generate code for the func if it does not have its body
        if func.body.is_none() {
            print!("declare ")
        } else {
            print!("define ")
        }

        // collect information about all variables including parameters
        let frame = compute_frame(self, func);
        self.push_frame(frame);

        let name = self.ctx.resolver.resolve_ident(&func.name).unwrap();
        let (_param_tys, ret_ty) = self
            .ctx
            .lookup_name_type(&name)
            .unwrap()
            .get_func_type()
            .unwrap();

        print!(
            "{} @{}(",
            self.ty_to_llty(&ret_ty).to_string(),
            func.name.symbol
        );

        // parameters
        let mut it = self
            .peek_frame()
            .get_locals()
            .iter()
            .filter(|(b, l)| b.kind == BindingKind::Arg && !l.reg.llty.is_void())
            .peekable();
        while let Some((_, local)) = it.next() {
            print!("{}", local.reg.to_string_with_type());
            if it.peek().is_some() {
                print!(", ");
            }
        }

        print!(")");

        let Some(body) = &func.body else{
            println!();
            return Ok(());
        };

        println!(" {{");

        // allocate local variables
        for (name_binding, local) in self.peek_frame().get_locals() {
            if name_binding.kind == BindingKind::Let && !local.reg.llty.is_void() {
                assert!(local.kind == LocalKind::Ptr);
                println!(
                    "\t{} = alloca {}",
                    local.reg.name,
                    local.reg.llty.peel_ptr().unwrap().to_string()
                );
            }
        }

        // allocate temporary variables
        for reg in self.peek_frame().get_ptrs_to_temporary().values() {
            println!(
                "\t{} = alloca {}",
                reg.name,
                reg.llty.peel_ptr().unwrap().to_string()
            );
        }

        let body_val = self.gen_block(body)?;

        if !self.ctx.get_type(body.id).is_never() {
            println!("\tret {}", body_val.to_string_with_type());
        }

        println!("}}");
        println!();

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
                self.eval_expr(expr)?;
                LLValue::Imm(LLImm::Void)
            }
            StmtKind::Expr(expr) => self.eval_expr(expr)?,
            StmtKind::Let(LetStmt { ident, ty: _, init }) => {
                let name = self.ctx.resolver.resolve_ident(ident).unwrap();
                let local = self.peek_frame().get_local(&name);

                if let Some(init) = init && local.kind == LocalKind::Ptr {
                    let ptr = self.gen_ident_lval(ident).unwrap();
                    // assign initializer
                    self.initialize_memory_with_value(&ptr, init)?;
                }
                LLValue::Imm(LLImm::Void)
            }
        };
        println!("; Finished stmt `{}`", stmt.span.to_snippet());
        Ok(val)
    }
}
