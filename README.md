# mini-rustc
**NOTE: This compiler is under development now**

mini-rustc a toy Rust compiler written in Rust from scratch.
This compiler implements typecheck but not other static analyses like lifetime or mutability.
If you find a bug, feel free to open an issue to report it!


mini-rustc has been much inspired by [GCC Rust](https://github.com/Rust-GCC/gccrs) and [Rui Ueyama's compiler book](https://www.sigbus.info/compilerbook).
Big thanks to these wonderful materials/software.


# Requirement
- x86-64 CPU
- Cargo

# Status
- [x] primitive types
    - `i32`, `bool`, unit(`()`), never(`!`), array(`[ty; N]`)
- [x] typechecking
- [ ] type inference
- [x] (nested) struct

## Build & Run
```
cargo build
cargo run <input>
```

## References
- https://github.com/Rust-GCC/gccrs
- https://www.sigbus.info/compilerbook
- https://github.com/rui314/chibicc/
