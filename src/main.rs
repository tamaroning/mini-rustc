#![feature(let_chains)]
mod ast;
mod codegen;
mod lexer;
mod parse;

use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: mini-rustc <input>");
        eprintln!("Invalid number of arguments");
        exit(1);
    }

    let lexer = lexer::Lexer::new(&args[1]);
    let mut parser = parse::Parser::new(lexer);
    let parse_result = parser.parse_crate();

    let Some(expr) = parse_result else {
        eprintln!("Failed to parse source code");
        exit(1);
    };

    let codegen_result = codegen::codegen(expr);
    let Ok(()) = codegen_result else {
        eprintln!("Failed to generate assembly");
        exit(1);
    };
}
