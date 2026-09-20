# Roadmap

Planned work for the C-Prime compiler. The current release is **1.0 (beta)**;
the items below target the next version. Order reflects expected impact.

## 1. Standard library growth

The standard library is the fastest way to make C-Prime useful for real
programs. Everything here is written in C-Prime itself (`examples/std/`).

- `String`: `split`, `trimmed`, `index_of`, `contains`, `starts_with`,
  `ends_with`, `to_upper`, `to_lower`, `compare`, `substring` — **done**
- `Vec[T]`: `pop`, `remove`, `insert`, `last` — `pop`/`remove`/`insert` **done**
- JSON parser (`examples/std/json.cp` + `examples/json_test.cp`) — **done**
- `HashSet[T]` over the existing hash-map machinery — **done** (`HashMapStr`,
  `HashMapI64`, `HashSetStr`, `HashSetI64` in `examples/std/hashmap.cp`)
- Buffered byte reader/writer for binary files
- Command-line arguments and environment (`getenv`-style) — **done**
  (`sys.cp`: `env_get`, `cmd_line`, file read/write, `Rng`)
- Time, random, sleep — **done** (`sys.cp`: `time_s`, `Rng`, `sleep_ms`)
- Small config-file (INI-style) parser — **done** (`ini.cp` + `ini_test.cp`)

## 2. Language features

- `Option[T]` / `Result[T, E]` in the standard library, with `?`-style error
  propagation (or `try`/`catch`)
- `match` on structs/tagged data and `enum` method support hardening
- Range-based `for` iterators over `Vec[T]` and `String` — **done**
- `{}`-style formatted printing (compile-time format-checked) on top of the
  variadic `printf` path — **done**

## 3. Compiler and diagnostics

- Error output with source snippets, highlighting, and suggestions — **done**
  (caret diagnostics for lex, parse, and typechecker errors)
- Structural sharing / fewer allocations in the typechecker and codegen
- Explicit compile-time constants evaluation for array sizes and match widths

## 4. Testing and release engineering

- Cargo-driven harness that compiles and runs every example: front-end
  `--check` in default CI, full codegen on a GitHub Actions runner with
  LLVM 22 — **done** (`tests/examples_tests.rs`, `.github/workflows/ci.yml`)
- CI builds a Windows compiler archive and attaches it to each release
  — **done** (`v*` tag workflow job)
- Benchmark suite for lexer/parser/codegen throughput

## 5. Ecosystem

- Editor syntax highlighting definitions — **done** (`editor/vscode`)
- `CONTRIBUTING.md`, issue templates, and a structured `docs/` folder
- Cookbook-style example programs: file copy tool, `wc`-style counter,
  `ls`-style directory listing, a text-based game — **done**
  (`examples/cookbook/`: `filecopy.cp`, `wc.cp`, `ls.cp`, `guessing.cp`)