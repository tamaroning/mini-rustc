mod codegen_crate;
mod codegen_expr;
mod codegen_utils;
mod frame;
mod llvm;

use self::frame::{Frame, LocalKind, VisitFrame};
use self::llvm::*;
use crate::ast::{self, Crate, Expr, ExprKind, Ident, NodeId};
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

    fn add_lladt(&mut self, name: &Rc<String>, lladt: LLAdtDef) {
        self.ll_adt_defs.insert(Rc::clone(name), lladt);
    }

    fn get_lladt(&self, name: &Rc<String>) -> Option<&LLAdtDef> {
        self.ll_adt_defs.get(name)
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
        let mut lladts = vec![];
        for (name, adt_def) in self.ctx.get_adt_defs() {
            let lladt = self.construct_lladt(&adt_def);
            lladts.push((Rc::clone(name), lladt));
        }
        for (name, lladt) in lladts {
            print!("%Struct.{} = type {{", name);
            for (i, (_, fd_llty)) in lladt.fields.iter().enumerate() {
                print!(" {}", fd_llty.to_string());
                if i != lladt.fields.len() - 1 {
                    print!(",");
                }
            }
            println!(" }}");
            self.add_lladt(&name, lladt);
        }

        println!();
        self.gen_crate(krate)?;
        // TODO: literals
        Ok(())
    }
}
