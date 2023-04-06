mod codegen_crate;
mod codegen_expr;
mod frame;

use self::frame::{Frame, VisitFrame};
use crate::ast::{self, Crate, Expr, ExprKind};
use crate::middle::ty::Ty;
use crate::middle::Ctxt;
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
    next_reg: usize,
}

impl<'a> Codegen<'a> {
    fn new(ctx: &'a mut Ctxt) -> Self {
        Codegen {
            ctx,
            current_frame: None,
            next_reg: 1,
        }
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
            Ty::Bool => LLTy::I8,
            _ => panic!(),
        }
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

    fn go(&mut self, krate: &'a Crate) -> Result<(), ()> {
        println!(r#"target triple = "x86_64-unknown-linux-gnu""#);
        println!("");
        self.gen_crate(krate)?;
        // TODO: literals
        Ok(())
    }

    fn to_ptr(&self, expr: &'a Expr) -> Result<Rc<LLReg>, ()> {
        match &expr.kind {
            ExprKind::Ident(ident) => {
                let name = self.ctx.resolver.resolve_ident(ident).unwrap();
                Ok(self.peek_frame().get_local(&name))
            }
            _ => panic!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LLTy {
    Void,
    I8,
    I32,
    Ptr(Box<LLTy>),
}

impl LLTy {
    fn to_string(&self) -> String {
        match self {
            LLTy::Void => "void".to_string(),
            LLTy::I8 => "i8".to_string(),
            LLTy::I32 => "i32".to_string(),
            LLTy::Ptr(inner) => format!("{}*", inner.to_string()),
        }
    }

    fn is_integer(&self) -> bool {
        matches!(self, LLTy::I32)
    }

    fn is_signed_integer(&self) -> bool {
        matches!(self, LLTy::I32)
    }

    fn peel_ptr(&self) -> &LLTy {
        match self {
            LLTy::Ptr(inner) => inner,
            _ => panic!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LLReg {
    reg: String,
    llty: LLTy,
}

impl LLReg {
    fn new(llreg: String, llty: LLTy) -> Self {
        LLReg { reg: llreg, llty }
    }
}
