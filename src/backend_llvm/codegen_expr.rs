use super::{Codegen, LLValue};
use crate::{
    ast::{self, Expr, ExprKind},
    backend_llvm::{LLImm, LLReg, LLTy},
};

impl<'a> Codegen<'a> {
    pub fn gen_expr(&mut self, expr: &'a Expr) -> Result<LLValue, ()> {
        println!("; Starts expr `{}`", expr.span.to_snippet());
        let ret: LLValue = match &expr.kind {
            ExprKind::NumLit(n) => {
                // FIXME: Panics in some cases
                let casted: i32 = (*n).try_into().unwrap();
                LLValue::Imm(LLImm::I32(casted))
            }
            ExprKind::BoolLit(b) => {
                if *b {
                    LLValue::Imm(LLImm::I8(1))
                } else {
                    LLValue::Imm(LLImm::I8(0))
                }
            }
            ExprKind::Unary(unop, inner) => match unop {
                ast::UnOp::Minus => {
                    let inner_val = self.gen_expr(inner)?;
                    assert!(inner_val.llty().is_integer());
                    let reg = self.get_fresh_reg();
                    println!(
                        "\t{reg} = sub {} 0, {}",
                        inner_val.llty().to_string(),
                        inner_val.to_string()
                    );
                    LLValue::Reg(LLReg::new(reg, inner_val.llty()))
                }
                ast::UnOp::Plus => self.gen_expr(inner)?,
            },
            ExprKind::Binary(binop, lhs, rhs) => {
                let l = self.gen_expr(lhs)?;
                let r = self.gen_expr(rhs)?;
                // checks if rhs and lhs have the same type
                assert_eq!(self.ctx.get_type(lhs.id), self.ctx.get_type(rhs.id));
                let rhs_lhs_llty = self.ty_to_llty(&self.ctx.get_type(lhs.id));

                let reg_name = self.get_fresh_reg();
                let llty = match binop {
                    ast::BinOp::Add => {
                        assert!(rhs_lhs_llty.is_integer());
                        println!(
                            "\t{reg_name} = add {} {}, {}",
                            rhs_lhs_llty.to_string(),
                            l.to_string(),
                            r.to_string()
                        );
                        rhs_lhs_llty
                    }
                    ast::BinOp::Sub => {
                        assert!(rhs_lhs_llty.is_integer());
                        println!(
                            "\t{reg_name} = sub {} {}, {}",
                            rhs_lhs_llty.to_string(),
                            l.to_string(),
                            r.to_string()
                        );
                        rhs_lhs_llty
                    }
                    ast::BinOp::Mul => {
                        assert!(rhs_lhs_llty.is_integer());
                        println!(
                            "\t{reg_name} = mul {} {}, {}",
                            rhs_lhs_llty.to_string(),
                            l.to_string(),
                            r.to_string()
                        );
                        rhs_lhs_llty
                    }
                    ast::BinOp::Eq => {
                        assert!(rhs_lhs_llty.is_integer());
                        println!(
                            "\t{reg_name} = icmp eq {}, {}",
                            l.to_string(),
                            r.to_string()
                        );
                        LLTy::I8
                    }
                    ast::BinOp::Ne => {
                        assert!(rhs_lhs_llty.is_integer());
                        println!(
                            "\t{reg_name} = icmp ne {}, {}",
                            l.to_string(),
                            r.to_string()
                        );
                        LLTy::I8
                    }
                    ast::BinOp::Gt => {
                        assert!(rhs_lhs_llty.is_signed_integer());
                        println!(
                            "\t{reg_name} = icmp sgt {}, {}",
                            l.to_string(),
                            r.to_string()
                        );
                        LLTy::I8
                    }
                    ast::BinOp::Lt => {
                        assert!(rhs_lhs_llty.is_signed_integer());
                        println!(
                            "\t{reg_name} = icmp slt {}, {}",
                            l.to_string(),
                            r.to_string()
                        );
                        LLTy::I8
                    }
                };
                LLValue::Reg(LLReg {
                    name: reg_name,
                    llty,
                })
            }
            ExprKind::Return(inner) => {
                let inner_val = self.gen_expr(inner)?;
                println!("\tret {}", inner_val.to_string_with_type());
                LLValue::Imm(LLImm::Void)
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
                    ptr_reg.name
                );
                LLValue::Reg(LLReg::new(reg, llty))
            }
            ExprKind::Assign(lhs, rhs) => {
                let rhs_val = self.gen_expr(rhs)?;
                let rhs_llty = self.ty_to_llty(&self.ctx.get_type(rhs.id));
                let lhs_ptr = self.to_ptr(lhs)?;
                println!(
                    "\tstore {} {}, {} {}",
                    rhs_llty.to_string(),
                    rhs_val.to_string(),
                    lhs_ptr.llty.to_string(),
                    lhs_ptr.name,
                );
                LLValue::Imm(LLImm::Void)
            }
            _ => panic!(),
        };

        println!("; Starts expr `{}`", expr.span.to_snippet());
        Ok(ret)
    }
}
