use crate::ast::{self, Block, Crate, Expr, ExprKind, Func, ItemKind, LetStmt, Stmt, StmtKind};
use crate::middle::ty::Ty;
use crate::middle::Ctxt;

pub fn compile(ctx: &mut Ctxt, krate: &Crate) -> Result<(), ()> {
    codegen(ctx, krate)?;

    Ok(())
}

pub fn codegen(ctx: &mut Ctxt, krate: &Crate) -> Result<(), ()> {
    let mut codegen = Codegen::new(ctx);
    codegen.go(krate)?;
    Ok(())
}

struct Codegen<'a> {
    ctx: &'a mut Ctxt,
    next_reg: usize,
}

impl<'a> Codegen<'a> {
    fn new(ctx: &'a mut Ctxt) -> Self {
        Codegen { ctx, next_reg: 1 }
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
        //println!("\tret void");
        println!("}}");
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
            StmtKind::Let(_) => {
                // TODO:
                todo!()
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
            _ => panic!(),
        };
        println!("; Starts expr `{}`", expr.span.to_snippet());
        Ok(ret)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum LLTy {
    Void,
    I8,
    I32,
}

impl LLTy {
    fn to_string(&self) -> String {
        match self {
            LLTy::Void => "void".to_string(),
            LLTy::I8 => "i8".to_string(),
            LLTy::I32 => "i32".to_string(),
        }
    }

    fn is_integer(&self) -> bool {
        matches!(self, LLTy::I32)
    }

    fn is_signed_integer(&self) -> bool {
        matches!(self, LLTy::I32)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct LLReg {
    reg: String,
    llty: LLTy,
}

impl LLReg {
    fn new(llreg: String, llty: LLTy) -> Self {
        LLReg { reg: llreg, llty }
    }
}
