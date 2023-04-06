mod frame;

use std::collections::HashMap;
use std::rc::Rc;

use crate::ast::{self, Block, Crate, Expr, ExprKind, Func, ItemKind, LetStmt, Stmt, StmtKind};
use crate::middle::ty::Ty;
use crate::middle::Ctxt;
use crate::resolve::NameBinding;

use self::frame::{Frame, VisitFrame};

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

    fn gen_crate(&mut self, krate: &'a Crate) -> Result<(), ()> {
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
            Some(LLReg { reg, llty }) => println!("\tret {} {}", llty.to_string(), reg),
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

    fn gen_block(&mut self, block: &'a Block) -> Result<Option<LLReg>, ()> {
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
            StmtKind::Let(LetStmt { ident, ty, init }) => {
                // if let stmt is in a loop, memory might be allocated inifinitely
                let name = self.ctx.resolver.resolve_ident(ident).unwrap();
                let reg = self.peek_frame().get_local(&name);
                println!("\t{} = alloca {}", reg.reg, reg.llty.peel_ptr().to_string());
                // TODO: init
                Ok(None)
            }
        };
        println!("; Finished stmt `{}`", stmt.span.to_snippet());
        res
    }

    fn gen_expr(&mut self, expr: &'a Expr) -> Result<Option<LLReg>, ()> {
        println!("; Starts expr `{}`", expr.span.to_snippet());
        let ret = match &expr.kind {
            ExprKind::NumLit(n) => {
                let reg = self.get_fresh_reg();
                let llty = LLTy::I32;
                println!("\t{reg} = bitcast i32 {} to i32", n);
                Some(LLReg::new(reg, llty))
            }
            ExprKind::BoolLit(b) => {
                let n = if *b { 1 } else { 0 };
                let reg = self.get_fresh_reg();
                let llty = LLTy::I8;
                println!("\t{reg} = bitcast i8 {} to i8", n);
                Some(LLReg { reg, llty })
            }
            ExprKind::Unary(unop, inner) => match unop {
                ast::UnOp::Minus => {
                    let llreg = self.gen_expr(inner)?.unwrap();
                    assert!(llreg.llty.is_integer());
                    let reg = self.get_fresh_reg();
                    println!("\t{reg} = sub {} 0, {}", llreg.llty.to_string(), llreg.reg);
                    Some(LLReg::new(reg, llreg.llty))
                }
                ast::UnOp::Plus => self.gen_expr(inner)?,
            },
            ExprKind::Binary(binop, lhs, rhs) => {
                let l = self.gen_expr(lhs)?.unwrap();
                let r = self.gen_expr(rhs)?.unwrap();
                // checks if rhs and lhs have the same type
                assert_eq!(self.ctx.get_type(lhs.id), self.ctx.get_type(rhs.id));
                let rhs_lhs_llty = self.ty_to_llty(&self.ctx.get_type(lhs.id));

                let reg = self.get_fresh_reg();
                let llty = match binop {
                    ast::BinOp::Add => {
                        assert!(rhs_lhs_llty.is_integer());
                        println!(
                            "\t{reg} = add {} {}, {}",
                            rhs_lhs_llty.to_string(),
                            l.reg,
                            r.reg
                        );
                        rhs_lhs_llty
                    }
                    ast::BinOp::Sub => {
                        assert!(rhs_lhs_llty.is_integer());
                        println!(
                            "\t{reg} = sub {} {}, {}",
                            rhs_lhs_llty.to_string(),
                            l.reg,
                            r.reg
                        );
                        rhs_lhs_llty
                    }
                    ast::BinOp::Mul => {
                        assert!(rhs_lhs_llty.is_integer());
                        println!(
                            "\t{reg} = mul {} {}, {}",
                            rhs_lhs_llty.to_string(),
                            l.reg,
                            r.reg
                        );
                        rhs_lhs_llty
                    }
                    ast::BinOp::Eq => {
                        assert!(rhs_lhs_llty.is_integer());
                        println!("\t{reg} = icmp eq {}, {}", l.reg, r.reg);
                        LLTy::I8
                    }
                    ast::BinOp::Ne => {
                        assert!(rhs_lhs_llty.is_integer());
                        println!("\t{reg} = icmp ne {}, {}", l.reg, r.reg);
                        LLTy::I8
                    }
                    ast::BinOp::Gt => {
                        assert!(rhs_lhs_llty.is_signed_integer());
                        println!("\t{reg} = icmp sgt {}, {}", l.reg, r.reg);
                        LLTy::I8
                    }
                    ast::BinOp::Lt => {
                        assert!(rhs_lhs_llty.is_signed_integer());
                        println!("\t{reg} = icmp slt {}, {}", l.reg, r.reg);
                        LLTy::I8
                    }
                };
                Some(LLReg { reg, llty })
            }
            ExprKind::Return(inner) => {
                let LLReg { reg, llty } = self.gen_expr(inner)?.unwrap();
                println!("\tret {} {}", llty.to_string(), reg);
                None
            }
            ExprKind::Block(block) => self.gen_block(block)?,
            ExprKind::Ident(_) => {
                let llty = self.ty_to_llty(&self.ctx.get_type(expr.id));
                let ptr_reg = self.to_ptr(expr)?;
                let reg = self.get_fresh_reg();
                //  %4 = load i32, i32* %2, align 4
                println!(
                    "\t{} = load {}, {} {}",
                    reg,
                    llty.to_string(),
                    ptr_reg.llty.to_string(),
                    ptr_reg.reg
                );
                Some(LLReg::new(reg, llty))
            }
            ExprKind::Assign(lhs, rhs) => {
                let rhs_reg = self.gen_expr(rhs)?.unwrap();
                let rhs_llty = self.ty_to_llty(&self.ctx.get_type(rhs.id));
                let lhs_ptr_reg = self.to_ptr(lhs)?;
                println!(
                    "\tstore {} {}, {} {}",
                    rhs_llty.to_string(),
                    rhs_reg.reg,
                    lhs_ptr_reg.llty.to_string(),
                    lhs_ptr_reg.reg,
                );
                None
            }
            _ => panic!(),
        };
        println!("; Starts expr `{}`", expr.span.to_snippet());
        Ok(ret)
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
