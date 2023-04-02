# mini-rustc

**NOTE: This compiler is under development now**

mini-rustc a toy Rust compiler written in Rust from scratch.
This compiler implements typecheck but not other static analyses like lifetime, mutability, or unsafety.
If you find a bug, feel free to open an issue to report it!

mini-rustc has been much inspired by [GCC Rust](https://github.com/Rust-GCC/gccrs) and [Rui Ueyama's compiler book](https://www.sigbus.info/compilerbook).
Big thanks to these wonderful materials/software.

## Requirement

- Cargo
- toolchains for x86-64 processor
  - necessary to generate executables

# Build & Run

To build mini-rustc, run the following command:

```sh
$ git clone <this repo>
$ cd mini-rustc
$ cargo build
```

To compile Rust code, run the following command:

```sh
$ cargo run <file>
```

or

```sh
$ cargo run <source>
```

## Test

Run the following command:

```rust
$ ./test.sh
```

## Compile Hello world!

`examples/hello.rs` contains:

```rust
extern "C" {
    fn printf(s: &str) -> i32;
}

fn main() -> () {
    unsafe {
        printf("Hello world!\n");
    };
}
```

Run the follwoing commands:

```sh
$ cargo run examples/hello.rs > tmp.s
$ gcc tmp.s -o a.out
$ ./a.out
Hello world!
```

# Status

- [x] types
  - `i32`, `bool`, unit(`()`), never(`!`), array(`[ty; N]`), `str`
  - [ ] references
  - ADT
    - [x] (nested) structs
    - [ ] enums
- [x] typechecking
- [ ] type inference
- items
  - [x] structs
  - [x] functions
    - return type cannot be omitted
    - struct params and returning structs are not supported
  - [x] `extern` blocks
  - [ ] modules
- statements
  - [x] let statement
    - `mut` and initializers are not supported
  - [x] expression statements
  - [x] return
- expressions
  - [x] arithmetic operators `+`, `-`, `*`
  - [x] comparison operators `==`
  - [x] literals: integer, boolean, string
  - [x] if-else expressions
  - [x] block expressions
- misc
  - [ ] paths
  - [ ] pattern matching

## Problem of ambiguous grammars

I have developed the parser refering to Rust Reference, but mini-rustc cannot parse several grammars correctly.
I will investigate rustc or other compilers to fix it.

examples:

```rust
// How do we decide condition is ident or struct expr?
fn main() -> i32 { if some_ident { 3 } else { 4 } }
// How do we decide this expr is a function call or two expr stmts?
fn main() -> i32 { () () }
```

## References

- https://github.com/Rust-GCC/gccrs
- https://www.sigbus.info/compilerbook
- https://github.com/rui314/chibicc/
