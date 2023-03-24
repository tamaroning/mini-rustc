use crate::ast::{BinOp, Expr, ExprKind, UnOp};

pub fn codegen(expr: &Expr) -> Result<(), ()> {
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");

    let Ok(()) = codegen_expr(expr) else {
        return Err(());
    };

    println!("\tpop rax");
    println!("\tret");

    Ok(())
}

fn codegen_expr(expr: &Expr) -> Result<(), ()> {
    match &expr.kind {
        ExprKind::NumLit(n) => {
            println!("\tpush {}", n);
        }
        ExprKind::Unary(unop, inner_expr) => {
            match unop {
                UnOp::Plus => return codegen_expr(&*inner_expr),
                UnOp::Minus => {
                    // compile `-expr`as `0 - expr`
                    println!("\tpush 0");
                    let Ok(()) = codegen_expr(inner_expr) else {
                        return Err(());
                    };
                    println!("\tpop rdi");
                    println!("\tpop rax");
                    println!("\tsub rax, rdi");
                    println!("\tpush rax");
                    return Ok(());
                }
            }
        }
        ExprKind::Binary(binop, lhs, rhs) => {
            let Ok(()) = codegen_expr(lhs) else {
                return Err(());
            };
            let Ok(()) = codegen_expr(rhs) else {
                return Err(());
            };
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
        }
        _ => todo!(),
    }
    Ok(())
}
