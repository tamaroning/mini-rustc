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
        eprintln!("Usage: mini-rustc (<input> or <file>)");
        eprintln!("Invalid number of arguments");
        exit(1);
    }

    // TODO: refine handling command line args
    let dump_enabled = args.contains(&"--dump".to_string());

    let path_or_src = args[1].clone();
    let src = if args[1].ends_with(".rs") {
        let res = std::fs::read_to_string(path_or_src);
        if let Ok(src) = res {
            src
        } else {
            eprintln!("Could not read file {}", args[1]);
            exit(1);
        }
    } else {
        path_or_src
    };

    let lexer = lexer::Lexer::new(&src);
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
        eprintln!("ICE: Failed to generate assembly");
        exit(1);
    };
}
