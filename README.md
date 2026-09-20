# C-Prime Compiler

C-Prime is a small systems programming language with a Rust-inspired syntax.
The compiler is written in Rust; the optional LLVM back end lowers typed AST
to machine code via clang.

## Status

The compiler works end to end: the programs in `examples/` compile to native
executables via the LLVM backend. A comprehensive self-checking test
(`examples/all_features.cp`) exercises every language feature and standard
library module.

## Language features

- C-like syntax with explicit memory management
- Structs with `impl` methods and generic types (`Vec[T]`, `Map[K, V]`, `Box[T]`)
- Enums (implicit and explicit values), type aliases, constants
- `match` (with literal and binding patterns) and `switch`/`case`
- `for` loops over `Vec[T]` and `String`
- Pointers (`*mut T`, `*const T`), `null`, `as` casts, `sizeOf`
- Format-checked printing with `{}` placeholders
- Standard library written in C-Prime itself (`examples/std/`): `Vec[T]`,
  `String`, `HashMap`, `HashSet`, `IntMap`, `StrIntMap`, INI parser, JSON
  parser, file I/O, system utilities (env, time, sleep, RNG)
- Direct C interop with `extern fn` (variadic supported)
- Core types: `i32`, `i64`, `f64`, `bool`, `char`, `usize`, arrays,
  and `*mut T` / `*const T` pointers

See `C-Prime-Guide.txt` and `C-Prime-Cheat-Sheet.txt` for the language.

## Requirements

- Rust toolchain (edition 2024, Rust 1.85+)
- LLVM 22.1+ (only for the `codegen` feature)

## Build

    cargo build                  # front end only
    cargo build --features codegen   # with LLVM codegen

Set `LLVM_SYS_221_PREFIX` to the LLVM install directory if not on PATH.

## Test

    cargo test                       # 199 front-end tests + integration
    cargo test --features codegen    # + 226 codegen tests + 16 example exe tests

## Run

    cargo run -- --check examples/demo.cp                       # typecheck
    cargo run -- --lex examples/demo.cp                         # dump tokens
    cargo run -- --parse examples/demo.cp                       # parse AST
    cargo run --features codegen -- examples/demo.cp --link -o demo.exe
    ./demo.exe

## Examples

- `beginner_example.cp`, `feature_demo.cp`, `advanced_demo.cp`, `demo.cp` —
  progressively larger showcases
- `all_features.cp` — comprehensive self-checking test of every feature
- `stdlib_demo.cp` — Vec, String, and file I/O combined
- `string_test.cp`, `vec_test.cp`, `map_test.cp`, `hashmap_test.cp`,
  `ini_test.cp`, `json_test.cp`, `sys_test.cp`, `io_test.cp` —
  standard library tests
- `wfreq.cp` — word-frequency counter
- `calculator.cp` — terminal calculator
- `calculator_gui.cp`, `tetris.cp`, `raylib_hello.cp` — raylib GUI demos
- `ffi_test.cp` — calling the C runtime
- `examples/cookbook/` — wc, filecopy, ls, guessing game

## Layout

    src/           compiler: lexer, parser, AST, symbol table, typechecker, LLVM codegen
    tests/         end-to-end integration tests driving the public API
    examples/      C-Prime programs, including the standard library
    editor/vscode/ VS Code extension for syntax highlighting
    C-Prime-Guide.txt, C-Prime-Cheat-Sheet.txt   language documentation

## License

Licensed under the Apache License, Version 2.0. See `LICENSE`.
