use std::process::exit;

fn main() {
    eprintln!("Usage: mini-rustc <input>");

    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Invalid number of arguments");
        exit(1);
    }
    println!(".intel_syntax noprefix");
    println!(".globl main");
    println!("main:");
    println!("\tmov rax, {}", args[1]);
    println!("\tret");
}
