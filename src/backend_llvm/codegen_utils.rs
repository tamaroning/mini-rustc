use std::rc::Rc;

use crate::{
    ast::{Expr, ExprKind, Ident},
    backend_llvm::llvm::LLTy,
};

use super::{frame::LocalKind, llvm::LLReg, Codegen};

impl<'a> Codegen<'a> {
    pub fn is_allocated(&self, expr: &'a Expr) -> bool {
        match &expr.kind {
            ExprKind::Ident(ident) => self.ident_is_allocated(ident),
            ExprKind::Index(array, _) => self.is_allocated(array),
            ExprKind::Field(strct, _) => self.is_allocated(strct),
            _ => todo!(),
        }
    }

    fn ident_is_allocated(&self, ident: &'a Ident) -> bool {
        let name = self.ctx.resolver.resolve_ident(ident).unwrap();
        let local = self.peek_frame().get_local(&name);
        local.kind == LocalKind::Ptr
    }

    pub fn gen_lval(&mut self, expr: &'a Expr) -> Result<Rc<LLReg>, ()> {
        match &expr.kind {
            ExprKind::Ident(ident) => self.gen_ident_lval(ident),
            ExprKind::Index(arr, index) => {
                let arr_ptr_reg = self.gen_lval(arr)?;
                let index_val = self.gen_expr(index)?;
                let new_reg = self.get_fresh_reg();

                println!(
                    "\t{} = getelementptr {}, {}, i32 0, {}",
                    new_reg,
                    arr_ptr_reg.llty.peel_ptr().unwrap().to_string(),
                    arr_ptr_reg.to_string_with_type(),
                    index_val.to_string_with_type()
                );
                // `[N x elem_ty]*` => `elem_ty*`
                let ret_llty = LLTy::Ptr(
                    arr_ptr_reg
                        .llty
                        .peel_ptr()
                        .unwrap()
                        .get_element_type()
                        .unwrap(),
                );
                Ok(LLReg::new(new_reg, Rc::new(ret_llty)))
            }
            ExprKind::Field(strct, field) => {
                let ty = self.ctx.get_type(strct.id);
                let adt_name = ty.get_adt_name().unwrap();
                let lladt = self.ll_adt_defs.get(adt_name).unwrap();
                let field_index = lladt.get_field_index(&field.symbol).unwrap();
                // `type { T1, T2, T3 }*` => `Tn*`
                let ret_llty = LLTy::Ptr(Rc::clone(&lladt.fields[field_index].1));

                let struct_ptr_reg = self.gen_lval(strct)?;

                let new_reg = self.get_fresh_reg();
                println!(
                    "\t{} = getelementptr {}, {}, i32 0, i32 {}",
                    new_reg,
                    struct_ptr_reg.llty.peel_ptr().unwrap().to_string(),
                    struct_ptr_reg.to_string_with_type(),
                    field_index
                );

                Ok(LLReg::new(new_reg, Rc::new(ret_llty)))
            }
            ExprKind::Struct(_, _) | ExprKind::Array(_) => {
                Ok(self.peek_frame().get_ptr_to_temporary(expr.id).unwrap())
            }
            _ => todo!(),
        }
    }

    pub fn gen_ident_lval(&self, ident: &'a Ident) -> Result<Rc<LLReg>, ()> {
        let name = self.ctx.resolver.resolve_ident(ident).unwrap();
        let local = self.peek_frame().get_local(&name);
        match &local.kind {
            LocalKind::Value => Err(()),
            LocalKind::Ptr => Ok(Rc::clone(&local.reg)),
        }
    }

    /*
    fn load(&self, ident: &'a Ident) -> Option<Rc<LLReg>> {
        let name = self.ctx.resolver.resolve_ident(ident).unwrap();
        let local = self.peek_frame().get_local(&name);
        match &local.kind {
            LocalKind::Value => Some(Rc::clone(&local.reg)),
            LocalKind::Ptr => None,
        }
    }
    */

    // ident is allocated on stack => load fromm its reg and returns the value
    // ident is not allocated => returns its reg
    pub fn load_ident_if_necessary(&mut self, ident: &'a Ident) -> Rc<LLReg> {
        let name = self.ctx.resolver.resolve_ident(ident).unwrap();
        let local = &self.peek_frame().get_local(&name);
        match &local.kind {
            LocalKind::Value => Rc::clone(&local.reg),
            LocalKind::Ptr => self.load_lval(&local.reg),
        }
    }

    pub fn load_lval(&mut self, reg: &Rc<LLReg>) -> Rc<LLReg> {
        let new_reg = self.get_fresh_reg();
        let derefed_ty = reg.llty.peel_ptr().unwrap();
        println!(
            "\t{} = load {}, {} {}",
            new_reg,
            derefed_ty.to_string(),
            reg.llty.to_string(),
            reg.name
        );
        LLReg::new(new_reg, derefed_ty)
    }

    pub fn mem_copy(&mut self, dist: &Rc<LLReg>, src: &Rc<LLReg>) {
        assert_eq!(dist.llty, src.llty);
        match &*src.llty {
            LLTy::Adt(name) => {
                let lladt = self.get_lladt(name).unwrap();
                for (i, (_, fd_llty)) in lladt.fields.iter().enumerate() {
                    todo!()
                }
            }
            _ => todo!(),
        }
    }
}
