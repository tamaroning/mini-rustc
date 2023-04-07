# mini-rustc

**NOTE: This compiler is under development now**

mini-rustc a toy Rust compiler written in Rust from scratch which outputs [LLVM IR](https://llvm.org/).
This compiler implements typecheck but not other static analyses like lifetime, mutability, or unsafety.
If you find a bug, feel free to open an issue to report it!

mini-rustc has been much inspired by [GCC Rust](https://github.com/Rust-GCC/gccrs) and [Rui Ueyama's compiler book](https://www.sigbus.info/compilerbook).
Big thanks to these wonderful materials/software.

## Requirement

- Cargo

Also, [llc](https://llvm.org/docs/CommandGuide/llc.html) is required to compile [LLVM IR](https://llvm.org/) to executables.

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

Generated LLVM IR is output to stdout.

## Test

Run the following command:

```rust
$ ./test.sh
```

## Compile Hello world!

`examples/hello.rs` contains:

```rust
extern "C" {
    fn puts(s: &str) -> i32;
}

fn main() -> () {
    unsafe {
        puts("Hello mini-rustc!");
    };
}
```

Run the follwoing commands:

```sh
$ cargo run examples/hello.rs > tmp.ll
$ llc tmp.ll -o tmp.s -opaque-pointers # this option is required!
$ gcc tmp.s -o a.out
$ ./a.out
Hello mini-rustc!
```

# Status

- Type system
  - Primitives `i32`, `bool`, unit(`()`), never(`!`), `str`
  - References
    - [x] `&'static str`
  - [x] Srrays
  - ADTs
    - [x] (Nested) Structs
    - [ ] Enums
  - [x] Typechecking
  - [ ] Type inference
- items
  - [x] Structs
  - [x] Functions
    - Return type cannot be omitted
    - Struct params and returning structs are not supported
  - [x] `extern` blocks
  - [ ] Modules
  - [ ] Global variables
- statements
  - [x] `let` statement
    - Keyword `mut` is not supported
  - [x] Expression statements
  - [x] Expression with `;`
- expressions
  - [x] Arithmetic operators `+`, `-`, `*`
  - [x] Comparison operators `==`, `<`, `>`
  - [x] Literals: integer, boolean, string
  - [ ] `if-else` expressions
  - [x] Block expressions `{ ... }`
  - [x] Return expressions `return expr`
    - Omitting expression is not supported (i.e. Require `return ()` for `return`)
  - [x] Call expressions `func(params...)`
    - Parameter passing: ZSTs and ADTs are supported
    - Return value: ADTs and arrays are not supported
  - [ ] Array expressions `[expr, expr, ...]`
  - [x] Struct expressions `SomeName { field1: expr, .. }`
  - [x] Field expressions `strct.field`
  - [x] Index expressions `array[index]`
- misc
  - [ ] Paths
  - [ ] Patterns (Pattern matching)
  - [x] Comments `//`

## ABI

mini-rustc's ABI is similar to system V ABI, but not fully compatible.
When functions are called, arrays and ADTs are passed via memory, ZST parameters are ignored (not passed).

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
