mod codegen_crate;
mod codegen_expr;
mod frame;

use self::frame::{Frame, Local, LocalKind, VisitFrame};
use crate::ast::{self, Crate, Expr, ExprKind, Ident};
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

    pub fn is_allocated(&self, expr: &'a Expr) -> bool {
        match &expr.kind {
            ExprKind::Ident(ident) => self.ident_is_allocated(ident),
            _ => todo!(),
        }
    }

    fn ident_is_allocated(&self, ident: &'a Ident) -> bool {
        let name = self.ctx.resolver.resolve_ident(ident).unwrap();
        let local = self.peek_frame().get_local(&name);
        local.kind == LocalKind::Ptr
    }

    fn get_addr(&self, expr: &'a Expr) -> Option<Rc<LLReg>> {
        match &expr.kind {
            ExprKind::Ident(ident) => self.get_ident_addr(ident),
            _ => todo!(),
        }
    }

    fn get_ident_addr(&self, ident: &'a Ident) -> Option<Rc<LLReg>> {
        let name = self.ctx.resolver.resolve_ident(ident).unwrap();
        let local = self.peek_frame().get_local(&name);
        match &local.kind {
            LocalKind::Value => None,
            LocalKind::Ptr => Some(Rc::clone(&local.reg)),
        }
    }

    fn get_ident_val(&self, ident: &'a Ident) -> Option<Rc<LLReg>> {
        let name = self.ctx.resolver.resolve_ident(ident).unwrap();
        let local = self.peek_frame().get_local(&name);
        match &local.kind {
            LocalKind::Value => Some(Rc::clone(&local.reg)),
            LocalKind::Ptr => None,
        }
    }

    // ident is allocated on stack => load fromm its reg and returns the value
    // ident is not allocated => returns its reg
    fn load_ident(&mut self, ident: &'a Ident) -> Rc<LLReg> {
        let name = self.ctx.resolver.resolve_ident(ident).unwrap();
        let local = &self.peek_frame().get_local(&name);
        match &local.kind {
            LocalKind::Value => Rc::clone(&local.reg),
            LocalKind::Ptr => {
                let reg = self.get_fresh_reg();
                let derefed_ty = local.reg.llty.peel_ptr().unwrap();
                println!(
                    "\t{} = load {}, {} {}",
                    reg,
                    derefed_ty.to_string(),
                    local.reg.llty.to_string(),
                    local.reg.name
                );
                LLReg::new(reg, derefed_ty)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LLTy {
    Void,
    I8,
    I32,
    Ptr(Rc<LLTy>),
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

    fn peel_ptr(&self) -> Option<Rc<LLTy>> {
        match self {
            LLTy::Ptr(inner) => Some(Rc::clone(inner)),
            _ => None,
        }
    }
}

pub enum LLValue {
    Reg(Rc<LLReg>),
    Imm(LLImm),
}

impl LLValue {
    pub fn to_string(&self) -> String {
        match self {
            LLValue::Reg(reg) => reg.name.clone(),
            LLValue::Imm(imm) => imm.to_string(),
        }
    }

    pub fn llty(&self) -> Rc<LLTy> {
        match self {
            LLValue::Reg(reg) => Rc::clone(&reg.llty),
            LLValue::Imm(imm) => imm.llty(),
        }
    }

    pub fn to_string_with_type(&self) -> String {
        match self {
            LLValue::Reg(reg) => reg.to_string_with_type(),
            LLValue::Imm(imm) => imm.to_string_with_type(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct LLReg {
    name: String,
    llty: Rc<LLTy>,
}

impl LLReg {
    fn new(name: String, llty: Rc<LLTy>) -> Rc<Self> {
        Rc::new(LLReg { name, llty })
    }

    pub fn to_string_with_type(&self) -> String {
        format!("{} {}", self.llty.to_string(), self.name)
    }
}

pub enum LLImm {
    I32(i32),
    I8(i8),
    Void,
}

impl LLImm {
    pub fn to_string(&self) -> String {
        match self {
            LLImm::I32(n) => format!("{n}"),
            LLImm::I8(n) => format!("{n}"),
            LLImm::Void => "void".to_string(),
        }
    }

    pub fn to_string_with_type(&self) -> String {
        match self {
            LLImm::I32(n) => format!("i32 {n}"),
            LLImm::I8(n) => format!("i8 {n}"),
            LLImm::Void => "void".to_string(),
        }
    }

    pub fn llty(&self) -> Rc<LLTy> {
        Rc::new(match self {
            LLImm::I32(_) => LLTy::I32,
            LLImm::I8(_) => LLTy::I8,
            LLImm::Void => LLTy::Void,
        })
    }
}
