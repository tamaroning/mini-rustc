#![feature(let_chains)]
mod analysis;
mod ast;
mod backend;
mod lexer;
mod parse;
mod ty;
mod typeck;

use std::process::exit;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: mini-rustc <input>");
        eprintln!("Invalid number of arguments");
        exit(1);
    }

    let dump_enabled = args.contains(&"--dump".to_string());

    let lexer = lexer::Lexer::new(&args[1]);
    let mut parser = parse::Parser::new(lexer);
    let parse_result = parser.parse_crate();

    let Some(krate) = parse_result else {
        eprintln!("Failed to parse source code");
        exit(1);
    };

    let mut ctx = analysis::Ctxt::new(dump_enabled);

    if ctx.dump_enabled {
        dbg!(&krate);
    }

    let typeck_result = typeck::typeck(&mut ctx, &krate);
    let Ok(()) = typeck_result else {
        if let Err(errors) = typeck_result {
            for e in errors {
                eprintln!("{}", e);
            }
        }
        eprintln!("Failed to typecheck crate");
        exit(1);
    };

    let codegen_result = backend::compile(&ctx, &krate);
    let Ok(()) = codegen_result else {
        eprintln!("Failed to generate assembly");
        exit(1);
    };
}
