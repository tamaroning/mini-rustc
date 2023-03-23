use crate::ast::{Expr, ExprKind};

pub fn codegen(expr: Expr) -> Result<(), ()> {
    let ExprKind::NumLit(n) = expr.kind;

    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    println!("\tmov rax, {}", n);
    println!("\tret");

    Ok(())
}
