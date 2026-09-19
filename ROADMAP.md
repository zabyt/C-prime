# Roadmap

Planned work for the C-Prime compiler. The current release is **1.0 (beta)**;
the items below target the next version. Order reflects expected impact.

## 1. Standard library growth

The standard library is the fastest way to make C-Prime useful for real
programs. Everything here is written in C-Prime itself (`examples/std/`).

- `String`: `split`, `trim`, `index_of`, `contains`, `starts_with`,
  `ends_with`, `to_upper`, `to_lower`, `compare`
- `Vec[T]`: `pop`, `remove`, `insert`, `contains`
- `HashSet[T]` over the existing hash-map machinery
- Buffered byte reader/writer for binary files
- Command-line arguments and environment (`getenv`-style)
- Time, random, sleep
- Small config-file (INI-style) parser

## 2. Language features

- `Option[T]` / `Result[T, E]` in the standard library, with `?`-style error
  propagation (or `try`/`catch`)
- `match` on structs/tagged data and `enum` method support hardening
- Range-based `for` iterators over `Vec[T]` and `String`
- `{}`-style formatted printing (compile-time format-checked) on top of the
  variadic `printf` path

## 3. Compiler and diagnostics

- Error output with source snippets, highlighting, and suggestions
- Structural sharing / fewer allocations in the typechecker and codegen
- Explicit compile-time constants evaluation for array sizes and match widths

## 4. Testing and release engineering

- Cargo-driven harness that compiles and runs every example: front-end
  `--check` in default CI, full codegen on a GitHub Actions runner with
  LLVM 22
- CI builds a Windows compiler archive and attaches it to each release
- Benchmark suite for lexer/parser/codegen throughput

## 5. Ecosystem

- Editor syntax highlighting definitions
- `CONTRIBUTING.md`, issue templates, and a structured `docs/` folder
- Cookbook-style example programs: file copy tool, `wc`-style counter,
  `ls`-style directory listing, a text-based game