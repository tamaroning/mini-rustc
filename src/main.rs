#![feature(let_chains)]
mod ast;
mod backend;
mod lexer;
mod middle;
mod parse;
mod resolve;
mod span;
mod typeck;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: mini-rustc (<input> or <file>)");
        eprintln!("Invalid number of arguments");
        std::process::exit(1);
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
            std::process::exit(1);
        }
    } else {
        path_or_src
    };

    let lexer = lexer::Lexer::new(src);
    let mut parser = parse::Parser::new(lexer);
    let parse_result = parser.parse_crate();

    let Some(krate) = parse_result else {
        eprintln!("Failed to parse source code");
        std::process::exit(1);
    };

    let mut ctx = middle::Ctxt::new(dump_enabled);

    if ctx.dump_enabled {
        dbg!(&krate);
    }

    ctx.resolve(&krate);

    if ctx.dump_enabled {
        dbg!(&ctx);
    }

    let typeck_result = typeck::typeck(&mut ctx, &krate);
    let Ok(()) = typeck_result else {
        if let Err(errors) = typeck_result {
            for e in errors {
                eprintln!("{}", e);
            }
        }
        eprintln!("Failed to typecheck crate");
        std::process::exit(1);
    };

    let codegen_result = backend::compile(&mut ctx, &krate);
    let Ok(()) = codegen_result else {
        eprintln!("ICE: Failed to generate assembly");
        std::process::exit(1);
    };
}
