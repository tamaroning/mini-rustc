use std::collections::HashMap;

use crate::analysis::Ctxt;
use crate::ast::{BinOp, Crate, Expr, ExprKind, Stmt, StmtKind, UnOp};
use crate::ty::Ty;

pub fn codegen(ctx: &Ctxt, krate: &Crate) -> Result<(), ()> {
    println!(".intel_syntax noprefix");
    println!(".globl main");

    codegen_main_func(ctx, krate)?;

    Ok(())
}

#[derive(Debug)]
struct FrameInfo<'ctx> {
    size: u32,
    locals: HashMap<&'ctx String, LocalInfo<'ctx>>,
}

#[derive(Debug)]
struct LocalInfo<'ctx> {
    offset: u32,
    size: u32,
    align: u32,
    ty: &'ctx Ty,
}

impl<'ctx> FrameInfo<'ctx> {
    fn new(ctx: &'ctx Ctxt) -> Self {
        let mut locals = HashMap::new();

        let mut current_ofsset: u32 = 0;
        for (sym, ty) in ctx.get_all_local_vars() {
            let local = LocalInfo {
                offset: current_ofsset,
                // assume size of type equals to 4
                size: 4,
                align: 4,
                ty,
            };
            locals.insert(sym, local);
            current_ofsset += 4;
        }
        FrameInfo {
            locals,
            size: current_ofsset,
        }
    }
}

fn codegen_main_func(ctx: &Ctxt, krate: &Crate) -> Result<(), ()> {
    let frame = FrameInfo::new(ctx);
    dbg!(&frame);

    println!("main:");
    codegen_func_prologue(&frame)?;
    // return 0 for empty body
    println!("\tmov rax, 0");
    codegen_stmts(&krate.stmts)?;
    codegen_func_epilogue(ctx, krate)?;
    Ok(())
}

fn codegen_func_prologue(frame: &FrameInfo) -> Result<(), ()> {
    println!("\tpush rbp");
    println!("\tmov rbp, rsp");
    println!("\tsub rsp, {}", frame.size);
    Ok(())
}

fn codegen_func_epilogue(ctx: &Ctxt, krate: &Crate) -> Result<(), ()> {
    println!("\tmov rsp, rbp");
    println!("\tpop rbp");
    println!("\tret");
    Ok(())
}

fn codegen_stmts(stmts: &Vec<Stmt>) -> Result<(), ()> {
    for stmt in stmts {
        codegen_stmt(stmt)?;
    }
    Ok(())
}

fn codegen_stmt(stmt: &Stmt) -> Result<(), ()> {
    match &stmt.kind {
        StmtKind::ExprStmt(expr) => {
            codegen_expr(expr)?;
            println!("\tpop rax");
            Ok(())
        }
        StmtKind::Let(_name) => Ok(()),
    }
}

fn codegen_expr(expr: &Expr) -> Result<(), ()> {
    match &expr.kind {
        ExprKind::NumLit(n) => {
            println!("\tpush {}", n);
            Ok(())
        }
        ExprKind::Unary(unop, inner_expr) => {
            match unop {
                UnOp::Plus => codegen_expr(inner_expr),
                UnOp::Minus => {
                    // compile `-expr`as `0 - expr`
                    println!("\tpush 0");
                    codegen_expr(inner_expr)?;
                    println!("\tpop rdi");
                    println!("\tpop rax");
                    println!("\tsub rax, rdi");
                    println!("\tpush rax");
                    Ok(())
                }
            }
        }
        ExprKind::Binary(binop, lhs, rhs) => {
            codegen_expr(lhs)?;
            codegen_expr(rhs)?;
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
    }
}
