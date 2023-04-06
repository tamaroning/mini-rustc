use super::Codegen;
use crate::{
    ast::{self, Expr, ExprKind},
    backend_llvm::{LLReg, LLTy},
};

impl<'a> Codegen<'a> {
    pub fn gen_expr(&mut self, expr: &'a Expr) -> Result<Option<LLReg>, ()> {
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
}
