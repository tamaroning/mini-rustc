use super::{Codegen, LLValue};
use crate::{
    ast::{self, Expr, ExprKind},
    backend_llvm::{llvm::LLConst, LLImm, LLReg, LLTy},
};
use std::rc::Rc;

impl<'gen, 'ctx> Codegen<'gen, 'ctx> {
    // evaluate expression
    // expr struct/array -> sturct*/array*
    // otherwise: expr: LLTY -> LLTY/void
    pub fn eval_expr(&mut self, expr: &'gen Expr) -> Result<LLValue, ()> {
        println!("; Starts expr `{}`", expr.span.to_snippet());
        let llty = self.ty_to_llty(&self.ctx.get_type(expr.id));
        if llty.eval_to_ptr() {
            return Ok(LLValue::Reg(self.gen_lval(expr)?));
        }

        let ret: LLValue = match &expr.kind {
            ExprKind::NumLit(n) => {
                // FIXME: Panics in some cases
                let casted: i32 = (*n).try_into().unwrap();
                LLValue::Imm(LLImm::I32(casted))
            }
            ExprKind::BoolLit(b) => {
                if *b {
                    LLValue::Imm(LLImm::I1(true))
                } else {
                    LLValue::Imm(LLImm::I1(false))
                }
            }
            ExprKind::Unit => LLValue::Imm(LLImm::Void),
            ExprKind::StrLit(s) => {
                let llcons = Rc::new(LLConst {
                    name: self.get_fresh_str_name(),
                    string_lit: s.clone(),
                    // FIXME: +1 for \00
                    llty: Rc::new(LLTy::Array(Rc::new(LLTy::I8), s.len() + 1)),
                });
                self.constants.push(Rc::clone(&llcons));
                LLValue::PtrConst(llcons)
            }
            ExprKind::Unary(unop, inner) => match unop {
                ast::UnOp::Minus => {
                    let inner_val = self.eval_expr(inner)?;
                    assert!(inner_val.llty().is_integer());
                    let reg = self.peek_frame_mut().get_fresh_reg();
                    println!(
                        "\t{reg} = sub {} 0, {}",
                        inner_val.llty().to_string(),
                        inner_val.to_string()
                    );
                    LLValue::Reg(LLReg::new(reg, inner_val.llty()))
                }
                ast::UnOp::Plus => self.eval_expr(inner)?,
            },
            ExprKind::Binary(binop, lhs, rhs) => {
                let l = self.eval_expr(lhs)?;
                let r = self.eval_expr(rhs)?;
                // checks if rhs and lhs have the same type
                assert_eq!(self.ctx.get_type(lhs.id), self.ctx.get_type(rhs.id));
                let rhs_lhs_llty = self.ty_to_llty(&self.ctx.get_type(lhs.id));

                let reg_name = self.peek_frame_mut().get_fresh_reg();
                let llty = match binop {
                    ast::BinOp::Add => {
                        assert!(rhs_lhs_llty.is_integer());
                        println!(
                            "\t{reg_name} = add {}, {}",
                            l.to_string_with_type(),
                            r.to_string()
                        );
                        rhs_lhs_llty
                    }
                    ast::BinOp::Sub => {
                        assert!(rhs_lhs_llty.is_integer());
                        println!(
                            "\t{reg_name} = sub {}, {}",
                            l.to_string_with_type(),
                            r.to_string()
                        );
                        rhs_lhs_llty
                    }
                    ast::BinOp::Mul => {
                        assert!(rhs_lhs_llty.is_integer());
                        println!(
                            "\t{reg_name} = mul {}, {}",
                            l.to_string_with_type(),
                            r.to_string()
                        );
                        rhs_lhs_llty
                    }
                    ast::BinOp::Eq => {
                        assert!(rhs_lhs_llty.is_integer());
                        println!(
                            "\t{reg_name} = icmp eq {}, {}",
                            l.to_string_with_type(),
                            r.to_string()
                        );
                        LLTy::I1
                    }
                    ast::BinOp::Ne => {
                        assert!(rhs_lhs_llty.is_integer());
                        println!(
                            "\t{reg_name} = icmp ne {}, {}",
                            l.to_string_with_type(),
                            r.to_string()
                        );
                        LLTy::I1
                    }
                    ast::BinOp::Gt => {
                        assert!(rhs_lhs_llty.is_signed_integer());
                        println!(
                            "\t{reg_name} = icmp sgt {}, {}",
                            l.to_string_with_type(),
                            r.to_string()
                        );
                        LLTy::I1
                    }
                    ast::BinOp::Lt => {
                        assert!(rhs_lhs_llty.is_signed_integer());
                        println!(
                            "\t{reg_name} = icmp slt {}, {}",
                            l.to_string_with_type(),
                            r.to_string()
                        );
                        LLTy::I1
                    }
                };
                LLValue::Reg(LLReg::new(reg_name, Rc::new(llty)))
            }
            ExprKind::Return(inner) => {
                let inner_val = self.eval_expr(inner)?;
                println!("\tret {}", inner_val.to_string_with_type());
                LLValue::Imm(LLImm::Void)
            }
            ExprKind::Block(block) => self.gen_block(block)?,
            // identifiers may not be allocated on memory
            ExprKind::Path(path) => LLValue::Reg(self.load_path(path)?),
            // arrays and structs are always allocated on memory
            ExprKind::Index(_, _) | ExprKind::Field(_, _) => {
                let lval = self.gen_lval(expr)?;
                let rval = self.load_ptr(&lval)?;
                LLValue::Reg(rval)
            }
            ExprKind::Assign(lhs, rhs) => {
                let rhs_llty = self.ty_to_llty(&self.ctx.get_type(rhs.id));

                if rhs_llty.eval_to_ptr() {
                    let lhs_ptr = self.gen_lval(lhs)?;
                    let rhs_ptr = self.gen_lval(rhs)?;
                    self.memcpy(&lhs_ptr, &rhs_ptr);
                } else {
                    let rhs_val = self.eval_expr(rhs)?;
                    let lhs_ptr = self.gen_lval(lhs).unwrap();

                    println!(
                        "\tstore {}, {} {}",
                        rhs_val.to_string_with_type(),
                        lhs_ptr.llty.to_string(),
                        lhs_ptr.name,
                    );
                }

                LLValue::Imm(LLImm::Void)
            }
            ExprKind::Call(func, args) => {
                let ExprKind::Path(path) = &func.kind else {
                    // TODO:
                    todo!();
                };

                let mut arg_vals = vec![];
                for arg in args {
                    let arg_ty = &self.ctx.get_type(arg.id);
                    let llty = self.ty_to_llty(arg_ty);
                    if !llty.is_void() {
                        let arg_val = self.eval_expr(arg)?;
                        arg_vals.push(arg_val);
                    }
                }

                let ret_llty = self.ty_to_llty(&self.ctx.get_type(expr.id));

                // instructions returning void cannot have a reg name
                print!("\t");
                let reg_name = if !ret_llty.is_void() {
                    let r = self.peek_frame_mut().get_fresh_reg();
                    print!("{} = ", r);
                    Some(r)
                } else {
                    None
                };

                let binding = self.ctx.resolve_path(path).unwrap();
                print!(
                    "call {} @{}(",
                    ret_llty.to_string(),
                    binding.cpath.demangle()
                );
                for (i, arg_val) in arg_vals.iter().enumerate() {
                    if !arg_val.llty().is_void() {
                        print!("{}", arg_val.to_string_with_type());
                        if i != arg_vals.len() - 1 {
                            print!(", ");
                        }
                    }
                }
                println!(")");

                if let Some(reg_name) = reg_name {
                    LLValue::Reg(LLReg::new(reg_name, Rc::new(ret_llty)))
                } else {
                    LLValue::Imm(LLImm::Void)
                }
            }
            ExprKind::If(cond, then, els) => {
                let res = self.gen_if_expr(cond, then, els)?;
                LLValue::Reg(res.0)
            }
            ExprKind::Struct(..) | ExprKind::Array(..) => panic!("ICE"),
        };

        println!("; Finishes expr `{}`", expr.span.to_snippet());
        Ok(ret)
    }

    /// Generate code for if expression. Returns the label of the last bb.
    pub fn gen_if_expr(
        &mut self,
        cond: &'gen Expr,
        then: &'gen Expr,
        els: &'gen Option<Box<Expr>>,
    ) -> Result<(Rc<LLReg>, String), ()> {
        let cond = self.eval_expr(cond)?;
        let then_label = self.get_fresh_label_name();
        let endif_label = self.get_fresh_label_name();
        let mut else_result = None;
        // if `else if` is found, this contains its endif label
        let mut else_label = None;

        if let Some(els) = els {
            else_label = Some(self.get_fresh_label_name());
            println!(
                "\tbr {}, label %{}, label %{}",
                cond.to_string_with_type(),
                then_label,
                else_label.as_ref().unwrap()
            );
            // else_label:
            println!("{}:\t; Else", else_label.as_ref().unwrap());
            // else block
            else_result = match &els.kind {
                ExprKind::If(cond2, then2, else2) => {
                    let res = self.gen_if_expr(cond2, then2, else2)?;
                    else_label = Some(res.1);
                    Some(LLValue::Reg(res.0))
                }
                ExprKind::Block(_) => Some(self.eval_expr(els)?),
                _ => panic!("ICE: else must be if expr or block expr"),
            };
            println!("\tbr label %{}", endif_label);
        } else {
            println!(
                "\tbr {}, label %{}, label %{}",
                cond.to_string_with_type(),
                then_label,
                endif_label
            );
        }
        // then_label:
        println!("{}:\t;Then", then_label);
        // then block
        let then_result = self.eval_expr(then)?;
        println!("\tbr label %{}", endif_label);

        println!("{}:\t; Endif", endif_label);
        if let Some(else_result) = else_result {
            let reg_name = self.peek_frame_mut().get_fresh_reg();
            println!(
                "\t{} = phi {} [{}, %{}], [{}, %{}]",
                reg_name,
                then_result.llty().to_string(),
                then_result.to_string(),
                then_label,
                else_result.to_string(),
                else_label.as_ref().unwrap(),
            );
            Ok((LLReg::new(reg_name, then_result.llty()), endif_label))
        } else {
            let reg_name = self.peek_frame_mut().get_fresh_reg();
            println!(
                "\t{} = phi {} [{}, %{}]",
                reg_name,
                then_result.llty().to_string(),
                then_result.to_string(),
                then_label,
            );
            Ok((LLReg::new(reg_name, then_result.llty()), endif_label))
        }
    }
}
