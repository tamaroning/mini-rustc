use crate::analysis::Ctxt;
use crate::ast::{BinOp, Crate, Expr, ExprKind, Stmt, StmtKind, UnOp};

pub fn codegen(ctx: &Ctxt, krate: &Crate) -> Result<(), ()> {
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    codegen_stmts(&krate.stmts)?;

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
