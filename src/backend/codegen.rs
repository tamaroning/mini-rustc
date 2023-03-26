use crate::ast::{BinOp, Crate, Expr, ExprKind, Stmt, StmtKind, UnOp};
use std::collections::HashMap;

use super::BackendCtxt;

pub fn codegen(bctx: &BackendCtxt, krate: &Crate) -> Result<(), ()> {
    let mut codegen = Codegen::new(bctx);
    codegen.codegen_crate(krate)?;
    Ok(())
}

struct Codegen<'a, 'ctx> {
    bctx: &'a BackendCtxt<'a, 'ctx>,
    current_frame: Option<FrameInfo<'a>>,
}

#[derive(Debug)]
struct FrameInfo<'a> {
    size: u32,
    locals: HashMap<&'a String, LocalInfo>,
}

#[derive(Debug)]
struct LocalInfo {
    offset: u32,
    size: u32,
    // align: u32,
}

impl<'ctx> FrameInfo<'ctx> {
    fn new(bctx: &'ctx BackendCtxt) -> Self {
        let mut locals = HashMap::new();

        let mut current_ofsset: u32 = 16;
        for sym in &bctx.locals {
            let local = LocalInfo {
                offset: current_ofsset,
                // assume size of type equals to 8
                size: 8,
            };
            locals.insert(*sym, local);
            current_ofsset += 8;
        }
        FrameInfo {
            locals,
            size: current_ofsset,
        }
    }

    fn get_local_info(&self, symbol: &String) -> Option<&LocalInfo> {
        self.locals.get(symbol)
    }
}

impl<'a: 'ctx, 'ctx> Codegen<'a, 'ctx> {
    fn new(bctx: &'a BackendCtxt<'a, 'ctx>) -> Self {
        Codegen {
            bctx,
            current_frame: None,
        }
    }

    fn push_current_frame(&mut self, frame: FrameInfo<'ctx>) {
        self.current_frame = Some(frame);
    }

    fn get_current_frame(&self) -> &FrameInfo {
        let Some(f) = &self.current_frame else {
            panic!("ICE");
        };
        f
    }

    fn pop_current_frame(&mut self) {
        if self.current_frame.is_none() {
            panic!("ICE: cannot pop the current frame");
        }
        self.current_frame = None;
    }

    fn codegen_crate(&mut self, krate: &Crate) -> Result<(), ()> {
        println!(".intel_syntax noprefix");
        println!(".globl main");
        self.codegen_main_func(krate)?;
        Ok(())
    }

    fn codegen_main_func(&mut self, krate: &Crate) -> Result<(), ()> {
        let frame = FrameInfo::new(self.bctx);
        if self.bctx.ctx.dump_enabled {
            dbg!(&frame);
        }
        self.push_current_frame(frame);

        println!("main:");
        self.codegen_func_prologue()?;
        // return 0 for empty body
        println!("\tmov rax, 0");
        self.codegen_stmts(&krate.stmts)?;
        self.codegen_func_epilogue();

        self.pop_current_frame();
        Ok(())
    }

    fn codegen_func_prologue(&self) -> Result<(), ()> {
        let frame = self.get_current_frame();
        println!("\tpush rbp");
        println!("\tmov rbp, rsp");
        println!("\tsub rsp, {}", frame.size);
        Ok(())
    }

    fn codegen_func_epilogue(&self) {
        println!("\tmov rsp, rbp");
        println!("\tpop rbp");
        println!("\tret");
    }

    fn codegen_stmts(&self, stmts: &Vec<Stmt>) -> Result<(), ()> {
        for stmt in stmts {
            self.codegen_stmt(stmt)?;
        }
        Ok(())
    }

    fn codegen_stmt(&self, stmt: &Stmt) -> Result<(), ()> {
        match &stmt.kind {
            StmtKind::ExprStmt(expr) => {
                self.codegen_expr(expr)?;
                println!("\tpop rax");
                Ok(())
            }
            StmtKind::Let(_name) => Ok(()),
        }
    }

    fn codegen_expr(&self, expr: &Expr) -> Result<(), ()> {
        match &expr.kind {
            ExprKind::NumLit(n) => {
                println!("#lit");
                println!("\tpush {}", n);
                Ok(())
            }
            ExprKind::Unary(unop, inner_expr) => {
                println!("#unary");
                match unop {
                    UnOp::Plus => self.codegen_expr(inner_expr),
                    UnOp::Minus => {
                        // compile `-expr`as `0 - expr`
                        println!("\tpush 0");
                        self.codegen_expr(inner_expr)?;
                        println!("\tpop rdi");
                        println!("\tpop rax");
                        println!("\tsub rax, rdi");
                        println!("\tpush rax");
                        Ok(())
                    }
                }
            }
            ExprKind::Binary(binop, lhs, rhs) => {
                println!("#binary");
                self.codegen_expr(lhs)?;
                self.codegen_expr(rhs)?;
                println!("\tpop rdi");
                println!("\tpop rax");

                match binop {
                    BinOp::Add => {
                        println!("\tadd rax, rdi");
                    }
                    BinOp::Sub => {
                        println!("\tsub rax, rdi");
                    }
                    BinOp::Mul => {
                        // NOTE: Result of mul is stored to rax
                        println!("\tmul rdi");
                    }
                };
                println!("\tpush rax");
                Ok(())
            }
            ExprKind::Ident(_ident) => {
                println!("#ident");
                self.codegen_lval(expr)?;
                println!("\tpop rax");
                println!("\tmov rax, [rax]");
                println!("\tpush rax");
                Ok(())
            }
            ExprKind::Assign(lhs, rhs) => {
                println!("#assign");
                self.codegen_lval(lhs)?;
                self.codegen_expr(rhs)?;
                println!("\tpop rdi");
                println!("\tpop rax");
                println!("\tmov [rax], rdi");
                // TODO: unit typeなのでpushしない
                println!("\tpush rdi");
                Ok(())
            }
        }
    }

    fn codegen_lval(&self, expr: &Expr) -> Result<(), ()> {
        let ExprKind::Ident(ident) = &expr.kind else {
            eprintln!("Expected ident, but found {:?}", expr);
            return Err(());
        };
        let Some(local) = self.get_current_frame().get_local_info(&ident.symbol) else {
            eprintln!("Unknwon identifier: {}", ident.symbol);
            return Err(());
        };
        println!("#lval");
        println!("\tmov rax, rbp");
        println!("\tsub rax, {}", local.offset);
        println!("\tpush rax");
        Ok(())
    }
}
