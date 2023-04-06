mod codegen_crate;
mod codegen_expr;
mod frame;
mod llvm;

use self::frame::{Frame, LocalKind, VisitFrame};
use self::llvm::*;
use crate::ast::{self, Crate, Expr, ExprKind, Ident};
use crate::middle::ty::{AdtDef, Ty};
use crate::middle::Ctxt;
use std::collections::HashMap;
use std::rc::Rc;

pub fn compile(ctx: &mut Ctxt, krate: &Crate) -> Result<(), ()> {
    codegen(ctx, krate)?;

    Ok(())
}

pub fn codegen(ctx: &mut Ctxt, krate: &Crate) -> Result<(), ()> {
    let mut codegen = Codegen::new(ctx);
    codegen.go(krate)?;
    Ok(())
}

pub struct Codegen<'a> {
    ctx: &'a mut Ctxt,
    current_frame: Option<Frame>,
    ll_adt_defs: HashMap<Rc<String>, LLAdtDef>,
    next_reg: usize,
}

impl<'a> Codegen<'a> {
    fn new(ctx: &'a mut Ctxt) -> Self {
        Codegen {
            ctx,
            current_frame: None,
            ll_adt_defs: HashMap::new(),
            next_reg: 1,
        }
    }

    fn get_fresh_reg(&mut self) -> String {
        let i = self.next_reg;
        self.next_reg += 1;
        format!("%{i}")
    }

    fn reset_fresh_reg(&mut self) {
        self.next_reg = 1;
    }

    fn ty_to_llty(&self, ty: &Ty) -> LLTy {
        match ty {
            Ty::Unit => LLTy::Void,
            Ty::I32 => LLTy::I32,
            Ty::Bool => LLTy::I8,
            Ty::Array(elem_ty, n) => LLTy::Array(Rc::new(self.ty_to_llty(elem_ty)), *n),
            Ty::Adt(name) => LLTy::Adt(Rc::clone(name)),
            _ => panic!(),
        }
    }

    fn construct_lladt(&self, adt: &AdtDef) -> LLAdtDef {
        let mut fields = vec![];
        for (fd, fd_ty) in &adt.fields {
            fields.push((Rc::clone(fd), Rc::new(self.ty_to_llty(fd_ty))))
        }
        LLAdtDef { fields }
    }

    pub fn compute_frame(&mut self, func: &'a ast::Func) -> Frame {
        let mut frame = Frame::new();
        let mut analyzer = VisitFrame {
            codegen: self,
            frame: &mut frame,
        };
        ast::visitor::go_func(&mut analyzer, func);
        frame
    }

    fn push_frame(&mut self, frame: Frame) {
        self.current_frame = Some(frame);
    }

    fn peek_frame(&self) -> &Frame {
        let Some(f) = &self.current_frame else {
            panic!("ICE");
        };
        f
    }

    fn pop_frame(&mut self) {
        if self.current_frame.is_none() {
            panic!("ICE: cannot pop the current frame");
        }
        self.current_frame = None;
    }

    /// Generate code for top-level
    fn go(&mut self, krate: &'a Crate) -> Result<(), ()> {
        println!(r#"target triple = "x86_64-unknown-linux-gnu""#);
        println!();

        // register all ADTs
        for (name, adt_def) in self.ctx.get_adt_defs() {
            let lladt = self.construct_lladt(adt_def);
            print!("%Struct.{} = type {{", name);
            for (i, (_, fd_llty)) in lladt.fields.iter().enumerate() {
                print!(" {}", fd_llty.to_string());
                if i != lladt.fields.len() - 1 {
                    print!(",");
                }
            }
            println!(" }}");
            //%struct.Empty = type {}
            self.ll_adt_defs.insert(Rc::clone(name), lladt);
        }

        println!();
        self.gen_crate(krate)?;
        // TODO: literals
        Ok(())
    }

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

    fn gen_lval(&mut self, expr: &'a Expr) -> Result<Rc<LLReg>, ()> {
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
            _ => todo!(),
        }
    }

    fn gen_ident_lval(&self, ident: &'a Ident) -> Result<Rc<LLReg>, ()> {
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
    fn load_ident_if_necessary(&mut self, ident: &'a Ident) -> Rc<LLReg> {
        let name = self.ctx.resolver.resolve_ident(ident).unwrap();
        let local = &self.peek_frame().get_local(&name);
        match &local.kind {
            LocalKind::Value => Rc::clone(&local.reg),
            LocalKind::Ptr => self.load_lval(&local.reg),
        }
    }

    fn load_lval(&mut self, reg: &Rc<LLReg>) -> Rc<LLReg> {
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
}
