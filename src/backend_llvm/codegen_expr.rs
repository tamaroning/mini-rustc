use std::rc::Rc;

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
            ExprKind::Unit => LLValue::Imm(LLImm::Void),
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
                LLValue::Reg(LLReg::new(reg_name, Rc::new(llty)))
            }
            ExprKind::Return(inner) => {
                let inner_val = self.gen_expr(inner)?;
                println!("\tret {}", inner_val.to_string_with_type());
                LLValue::Imm(LLImm::Void)
            }
            ExprKind::Block(block) => self.gen_block(block)?,
            // identifiers may not be allocated on memory
            ExprKind::Ident(ident) => LLValue::Reg(self.load_ident_if_necessary(ident)),
            // arrays and structs are always allocated on memory
            ExprKind::Index(_, _) | ExprKind::Field(_, _) => {
                let lval = self.gen_lval(expr)?;
                let rval = self.load_lval(&lval);
                LLValue::Reg(rval)
            }
            ExprKind::Assign(lhs, rhs) => {
                let rhs_val = self.gen_expr(rhs)?;

                if self.is_allocated(lhs) {
                    let lhs_addr_reg = self.gen_lval(lhs).unwrap();
                    println!(
                        "\tstore {}, {} {}",
                        rhs_val.to_string_with_type(),
                        lhs_addr_reg.llty.to_string(),
                        lhs_addr_reg.name,
                    );
                } else {
                    todo!()
                }

                LLValue::Imm(LLImm::Void)
            }
            ExprKind::Call(func, args) => {
                let ExprKind::Ident(ident) = &func.kind else {
                    todo!();
                };
                let mut arg_vals = vec![];
                for arg in args {
                    let arg_val = self.gen_expr(arg)?;
                    arg_vals.push(arg_val);
                }

                let ret_llty = self.ty_to_llty(&self.ctx.get_type(expr.id));

                // instructions returning void cannot have a reg name
                print!("\t");
                let reg_name = if !ret_llty.is_void() {
                    let r = self.get_fresh_reg();
                    print!("{} =", r);
                    Some(r)
                } else {
                    None
                };

                print!("call {} @{}(", ret_llty.to_string(), ident.symbol);
                for (i, arg_val) in arg_vals.iter().enumerate() {
                    print!("{}", arg_val.to_string_with_type());
                    if i != arg_vals.len() - 1 {
                        print!(", ");
                    }
                }
                print!(")");

                if let Some(reg_name) = reg_name {
                    LLValue::Reg(LLReg::new(reg_name, Rc::new(ret_llty)))
                } else {
                    LLValue::Imm(LLImm::Void)
                }
            }
            _ => panic!(),
        };

        println!("; Starts expr `{}`", expr.span.to_snippet());
        Ok(ret)
    }
}
