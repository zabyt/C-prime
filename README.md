# C-Prime Compiler

C-Prime is a small systems programming language with its own compiler. The
front end (lexer, parser, typechecker) is a dependency-free Rust crate; the
back end lowers the typed AST to LLVM IR and machine code.

## Status

The compiler works end to end: the programs in `examples/` compile to native
executables via the LLVM backend.

## Language features

- C-like syntax with explicit memory management
- Structs with `impl` methods and generic types (`Vec[T]`, `Map[K, V]`)
- Standard library written in C-Prime itself (`examples/std/`): `Vec[T]`,
  `String`, a hash map with string/pointer keys, file I/O, number formatting
  and parsing
- Direct C interop with `extern fn`, including variadic declarations
  (`malloc`, `printf`, `snprintf`, `fopen`, ...)
- Core types: `i32`, `i64`, `f64`, `bool`, `char`, `usize`, arrays, and
  `*mut T` / `*const T` pointers

See `C-Prime-Guide.txt` and `C-Prime-Cheat-Sheet.txt` for the language.

## Requirements

- A Rust toolchain that supports edition 2024 (Rust 1.85 or newer)
- LLVM 22.1+ — only for the `codegen` feature; the front end builds and tests
  without it

## Build

Front end only:

    cargo build

With LLVM code generation:

    cargo build --features codegen

The `codegen` feature requires LLVM 22.1 discoverable by llvm-sys. Set
`LLVM_SYS_221_PREFIX` to the directory containing `bin/llvm-config.exe` if it
is not already on `PATH`.

## Test

    cargo test
    cargo test --features codegen

## Run

Compile and run a program with the LLVM backend:

    cargo run -- --check examples/demo.cp                       # typecheck only
    cargo run -- --lex examples/demo.cp                         # dump tokens
    cargo run -- --parse examples/demo.cp                       # parse + AST summary
    cargo run -- --emit-llvm out.ll examples/demo.cp            # LLVM IR only
    cargo run --features codegen -- examples/demo.cp --link -o demo.exe
    ./demo.exe

## Examples

Programs in `examples/` (with the standard library under `examples/std/`):

- `beginner_example.cp`, `feature_demo.cp`, `advanced_demo.cp`, `demo.cp` —
  progressively larger showcases
- `stdlib_demo.cp` — `Vec[T]`, `String`, and file I/O combined
- `string_test.cp`, `vec_test.cp`, `map_test.cp`, `io_test.cp` —
  standard library exercises
- `wfreq.cp` — word-frequency counter reading `wfreq_input.txt`
- `calculator.cp` — terminal calculator; `calculator_gui.cp`, `tetris.cp`,
  `raylib_hello.cp` — raylib GUI demos (require `raylib.dll` at runtime)
- `ffi_test.cp` — calling the C runtime

## Layout

    src/           compiler: lexer, parser, AST, symbol table, typechecker, LLVM codegen
    tests/         end-to-end integration tests driving the public API
    examples/      C-Prime programs, including the standard library
    C-Prime-Guide.txt, C-Prime-Cheat-Sheet.txt   language documentation

## License

Licensed under the Apache License, Version 2.0. See `LICENSE`.