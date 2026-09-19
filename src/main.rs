






use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

use cprime_compiler::ast::Program;
use cprime_compiler::lexer::Lexer;
use cprime_compiler::parser::Parser as CpParser;
use cprime_compiler::token::Token;


#[derive(Debug, Parser)]
#[command(
    name = "cprime",
    version,
    about = "Compiler for the C-Prime (.cp) systems programming language",
    long_about = "cprime compiles C-Prime (.cp) source files to machine code via LLVM.\n\n\
        Phase 1 implements the CLI harness and lexer; Phase 2 adds parsing.\n\
        Use `cprime --lex FILE` to dump the token stream, or `cprime --parse FILE`\n\
        to validate and summarize the parsed AST."
)]
struct Cli {
    
    #[arg(value_name = "FILE")]
    input: PathBuf,

    
    #[arg(long)]
    lex: bool,

    
    #[arg(long)]
    parse: bool,

    
    #[arg(long)]
    check: bool,

    
    #[arg(short = 'o', long, value_name = "PATH")]
    output: Option<PathBuf>,

    
    #[arg(long, value_name = "PATH", requires = "output")]
    emit_llvm: Option<PathBuf>,

    
    #[arg(long, requires = "output")]
    link: bool,

    
    #[arg(long, value_name = "NAME")]
    link_lib: Vec<String>,

    
    #[arg(long, value_name = "DIR")]
    lib_path: Vec<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    run(cli)
}

fn run(cli: Cli) -> ExitCode {
    let source = match std::fs::read_to_string(&cli.input) {
        Ok(src) => src,
        Err(err) => {
            eprintln!("error: failed to read `{}`: {err}", cli.input.display());
            return ExitCode::FAILURE;
        }
    };

    
    
    let dir = cli
        .input
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| std::path::PathBuf::from("."));
    let mut stack = Vec::new();
    let source = match expand_includes(&source, &dir, &mut stack) {
        Ok(src) => src,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    let file_name = cli.input.display().to_string();
    let tokens = match Lexer::new(&source, file_name).tokenize() {
        Ok(tokens) => tokens,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    if cli.lex {
        dump_tokens(&tokens);
        return ExitCode::SUCCESS;
    }

    let mut parser = CpParser::new(tokens);
    let program = match parser.parse_program() {
        Ok(program) => program,
        Err(err) => {
            eprintln!("error: {err}");
            return ExitCode::FAILURE;
        }
    };

    if cli.parse {
        summarize_program(&program);
        return ExitCode::SUCCESS;
    }

    let errors = cprime_compiler::typechecker::check_program(&program);
    if !errors.is_empty() {
        for err in &errors {
            eprintln!("error: {err}");
        }
        eprintln!("error: type checking failed with {} error(s)", errors.len());
        return ExitCode::FAILURE;
    }

    if cli.check {
        println!("check succeeded: {} item(s), no semantic errors", program.items.len());
        return ExitCode::SUCCESS;
    }

    
    
    #[cfg(feature = "codegen")]
    if cli.output.is_some() || cli.emit_llvm.is_some() || cli.link {
        return emit(&program, &cli);
    }

    
    
    
    eprintln!("error: compilation is not yet implemented; use `cprime --lex FILE`, `cprime --parse FILE` or `cprime --check FILE` to inspect the front end");
    ExitCode::from(2)
}



#[cfg(feature = "codegen")]
fn emit(program: &Program, cli: &Cli) -> ExitCode {
    use cprime_compiler::codegen::{emit_llvm_ir_to_file, emit_object};
    let output = cli.output.clone().unwrap_or_else(|| {
        let mut p = cli.input.clone();
        p.set_extension("o");
        p
    });
    let mut ok = true;
    if let Some(path) = &cli.emit_llvm {
        match emit_llvm_ir_to_file(program, path) {
            Ok(()) => println!("wrote LLVM IR to `{}`", path.display()),
            Err(err) => {
                eprintln!("error: {err}");
                ok = false;
            }
        }
    }
    if ok && !cli.link {
        match emit_object(program, &output) {
            Ok(()) => println!("wrote object file to `{}`", output.display()),
            Err(err) => {
                eprintln!("error: {err}");
                ok = false;
            }
        }
    } else if ok && cli.link {
        ok = link_object(program, &output, &cli.link_lib, &cli.lib_path);
    }
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}







#[cfg(feature = "codegen")]
fn link_object(
    program: &Program,
    output: &std::path::Path,
    link_libs: &[String],
    lib_paths: &[std::path::PathBuf],
) -> bool {
    use cprime_compiler::codegen::emit_object;

    let obj = output.with_extension("cprime.o");
    if let Err(err) = emit_object(program, &obj) {
        eprintln!("error: {err}");
        return false;
    }

    
    let clang = find_clang();
    let mut cmd = std::process::Command::new(&clang);
    cmd.arg(&obj).arg("-o").arg(output);
    for dir in lib_paths {
        cmd.arg(format!("-L{}", dir.display()));
    }
    for lib in link_libs {
        cmd.arg(format!("-l{lib}"));
    }
    match cmd.output() {
        Ok(out) if out.status.success() => {
            let _ = std::fs::remove_file(&obj);
            println!("wrote executable to `{}`", output.display());
            true
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            eprintln!(
                "error: linking with `{}` failed:\n{}",
                clang.display(),
                stderr.trim_end()
            );
            let _ = std::fs::remove_file(&obj);
            false
        }
        Err(err) => {
            eprintln!("error: failed to run `{}`: {err}", clang.display());
            let _ = std::fs::remove_file(&obj);
            false
        }
    }
}



#[cfg(feature = "codegen")]
fn find_clang() -> std::path::PathBuf {
    if let Some(path) = std::env::var_os("LLVM_SYS_221_PREFIX") {
        let candidate = std::path::PathBuf::from(path).join("bin").join("clang.exe");
        if candidate.is_file() {
            return candidate;
        }
    }
    if let Some(path) = std::env::var_os("PATH") {
        for dir in std::env::split_paths(&path) {
            for name in ["clang.exe", "clang-22.exe", "clang"] {
                let candidate = dir.join(name);
                if candidate.is_file() {
                    return candidate;
                }
            }
        }
    }
    ["C:\\Program Files\\LLVM\\bin\\clang.exe", "C:\\LLVM\\bin\\clang.exe"]
        .iter()
        .map(std::path::PathBuf::from)
        .find(|p| p.is_file())
        .unwrap_or_else(|| "clang".into())
}




fn expand_includes(
    source: &str,
    base_dir: &std::path::Path,
    stack: &mut Vec<std::path::PathBuf>,
) -> Result<String, String> {
    let mut out = String::new();
    for (i, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("include") {
            let rest = rest.trim_start();
            if let Some(inner) = rest.strip_prefix('"') {
                if let Some((path, tail)) = inner.split_once('"') {
                    if tail.trim() == ";" {
                        let path = base_dir.join(path);
                        let canon = std::fs::canonicalize(&path)
                            .map_err(|_| format!("include: cannot open `{}`", path.display()))?;
                        if stack.contains(&canon) {
                            return Err(format!(
                                "include: recursive include of `{}`",
                                path.display()
                            ));
                        }
                        let text = std::fs::read_to_string(&path)
                            .map_err(|_| format!("include: cannot read `{}`", path.display()))?;
                        stack.push(canon);
                        let sub_dir = path
                            .parent()
                            .map(|p| p.to_path_buf())
                            .unwrap_or_else(|| std::path::PathBuf::from("."));
                        out.push_str(&expand_includes(&text, &sub_dir, stack)?);
                        stack.pop();
                        continue;
                    }
                }
            }
        }
        out.push_str(line);
        out.push('\n');
        let _ = i;
    }
    Ok(out)
}


fn dump_tokens(tokens: &[Token]) {
    for token in tokens {
        let loc = &token.span.start;
        println!("{}:{}:{}", loc.line, loc.column, token.kind.describe());
    }
}


fn summarize_program(program: &Program) {
    println!("parsed {} item(s) successfully:", program.items.len());
    for (i, item) in program.items.iter().enumerate() {
        let loc = item.span().start.clone();
        println!("  [{}] {} (at {}:{})", i, item.summary(), loc.line, loc.column);
    }
}
