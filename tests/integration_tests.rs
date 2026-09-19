





use cprime_compiler::lexer::Lexer;
use cprime_compiler::token::{
    IntegerLiteral, Keyword, LexError, Radix, SourceLocation, Symbol, TokenKind,
};

fn tokenize(src: &str) -> Vec<TokenKind> {
    Lexer::new(src, "integration.cp")
        .tokenize()
        .unwrap_or_else(|e| panic!("unexpected lex error: {e}"))
        .into_iter()
        .map(|t| t.kind)
        .collect()
}

#[test]
fn lexes_vector2_struct_program() {
    
    let src = r#"
struct Vector2 {
    x: f64,
    y: f64
}

impl Vector2 {
    fn new(x: f64, y: f64) -> Vector2 {
        return Vector2 { x: x, y: y };
    }
}
"#;
    let kinds = tokenize(src);

    use Keyword::*;
    use Symbol::*;
    let expected = vec![
        TokenKind::Keyword(Struct),
        TokenKind::Identifier("Vector2".into()),
        TokenKind::Symbol(LeftBrace),
        TokenKind::Identifier("x".into()),
        TokenKind::Symbol(Colon),
        TokenKind::Keyword(F64),
        TokenKind::Symbol(Comma),
        TokenKind::Identifier("y".into()),
        TokenKind::Symbol(Colon),
        TokenKind::Keyword(F64),
        TokenKind::Symbol(RightBrace),
        TokenKind::Keyword(Impl),
        TokenKind::Identifier("Vector2".into()),
        TokenKind::Symbol(LeftBrace),
        TokenKind::Keyword(Fn),
        TokenKind::Identifier("new".into()),
        TokenKind::Symbol(LeftParen),
        TokenKind::Identifier("x".into()),
        TokenKind::Symbol(Colon),
        TokenKind::Keyword(F64),
        TokenKind::Symbol(Comma),
        TokenKind::Identifier("y".into()),
        TokenKind::Symbol(Colon),
        TokenKind::Keyword(F64),
        TokenKind::Symbol(RightParen),
        TokenKind::Symbol(Arrow),
        TokenKind::Identifier("Vector2".into()),
        TokenKind::Symbol(LeftBrace),
        TokenKind::Keyword(Return),
        TokenKind::Identifier("Vector2".into()),
        TokenKind::Symbol(LeftBrace),
        TokenKind::Identifier("x".into()),
        TokenKind::Symbol(Colon),
        TokenKind::Identifier("x".into()),
        TokenKind::Symbol(Comma),
        TokenKind::Identifier("y".into()),
        TokenKind::Symbol(Colon),
        TokenKind::Identifier("y".into()),
        TokenKind::Symbol(RightBrace),
        TokenKind::Symbol(Semicolon),
        TokenKind::Symbol(RightBrace),
        TokenKind::Symbol(RightBrace),
        TokenKind::Eof,
    ];
    assert_eq!(kinds, expected);
}

#[test]
fn lexes_generics_and_pointer_syntax() {
    let kinds = tokenize("fn swap[T](a: *mut T, b: *mut T)");
    use Keyword::*;
    use Symbol::*;
    assert_eq!(
        kinds,
        vec![
            TokenKind::Keyword(Fn),
            TokenKind::Identifier("swap".into()),
            TokenKind::Symbol(LeftBracket),
            TokenKind::Identifier("T".into()),
            TokenKind::Symbol(RightBracket),
            TokenKind::Symbol(LeftParen),
            TokenKind::Identifier("a".into()),
            TokenKind::Symbol(Colon),
            TokenKind::Symbol(Star),
            TokenKind::Keyword(Mut),
            TokenKind::Identifier("T".into()),
            TokenKind::Symbol(Comma),
            TokenKind::Identifier("b".into()),
            TokenKind::Symbol(Colon),
            TokenKind::Symbol(Star),
            TokenKind::Keyword(Mut),
            TokenKind::Identifier("T".into()),
            TokenKind::Symbol(RightParen),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn lexes_match_pattern_matching() {
    let kinds = tokenize("match val { 1 => true, _ => false }");
    use Keyword::*;
    use Symbol::*;
    assert_eq!(
        kinds,
        vec![
            TokenKind::Keyword(Match),
            TokenKind::Identifier("val".into()),
            TokenKind::Symbol(LeftBrace),
            TokenKind::Integer(IntegerLiteral {
                value: 1,
                radix: Radix::Decimal
            }),
            TokenKind::Symbol(FatArrow),
            TokenKind::Keyword(True),
            TokenKind::Symbol(Comma),
            TokenKind::Symbol(Underscore),
            TokenKind::Symbol(FatArrow),
            TokenKind::Keyword(False),
            TokenKind::Symbol(RightBrace),
            TokenKind::Eof,
        ]
    );
}

#[test]
fn reports_errors_with_file_line_and_column() {
    let err = Lexer::new("\"oops", "broken.cp").tokenize().unwrap_err();
    assert_eq!(
        err,
        LexError::UnterminatedString {
            location: SourceLocation::new("broken.cp", 1, 1, 0)
        }
    );
    assert_eq!(err.to_string(), "broken.cp:1:1: unterminated string literal");
}

#[test]
#[allow(clippy::approx_constant)] 
fn float_literals_have_exact_values() {
    let kinds = tokenize("3.14 2.0 1.5e10");
    match &kinds[0] {
        TokenKind::Float(f) => assert_eq!(f.value, 3.14),
        other => panic!("expected float, got {other:?}"),
    }
    match &kinds[1] {
        TokenKind::Float(f) => assert_eq!(f.value, 2.0),
        other => panic!("expected float, got {other:?}"),
    }
    match &kinds[2] {
        TokenKind::Float(f) => assert_eq!(f.value, 1.5e10),
        other => panic!("expected float, got {other:?}"),
    }
}

#[test]
fn source_locations_are_exact_across_newlines() {
    let tokens = Lexer::new("let a = 1;\nlet b = 2;", "loc.cp").tokenize().unwrap();
    let (line, col, off) = (
        tokens[0].span.start.line,
        tokens[0].span.start.column,
        tokens[0].span.start.offset,
    );
    assert_eq!((line, col, off), (1, 1, 0));

    let b = &tokens[6];
    assert_eq!((b.span.start.line, b.span.start.column), (2, 5));
    assert_eq!(b.span.start.offset, 15);
}





use cprime_compiler::ast::{
    Expr, Item, LiteralExpr, MatchArmBody, Pattern, Stmt, Type,
};
use cprime_compiler::parser::{parse_tokens, ParseError};


fn parse(src: &str) -> cprime_compiler::ast::Program {
    let tokens = Lexer::new(src, "integration.cp").tokenize().unwrap();
    parse_tokens(tokens).unwrap_or_else(|e| panic!("unexpected parse error: {e}"))
}


fn parse_err(src: &str) -> ParseError {
    let tokens = Lexer::new(src, "integration.cp").tokenize().unwrap();
    match parse_tokens(tokens) {
        Ok(_) => panic!("expected parse error, got none"),
        Err(e) => e,
    }
}

#[test]
fn parses_spec_vector2_program() {
    let src = r#"
import std.io;

struct Vector2 {
    x: f64,
    y: f64
}

impl Vector2 {
    fn new(x: f64, y: f64) -> Vector2 {
        return Vector2 { x: x, y: y };
    }

    fn length(self) -> f64 {
        return sqrt(x * x + y * y);
    }
}

fn main() -> i32 {
    let v: Vector2 = Vector2::new(1.5, 2.5);
    return 0;
}
"#;
    let prog = parse(src);
    assert_eq!(prog.items.len(), 4);
    assert!(matches!(prog.items[0], Item::Import(_)));
    assert!(matches!(prog.items[1], Item::Struct(_)));
    match &prog.items[2] {
        Item::Impl(blk) => {
            assert_eq!(blk.type_name, "Vector2");
            assert_eq!(blk.methods.len(), 2);
            assert_eq!(blk.methods[0].name, "new");
            assert_eq!(blk.methods[1].name, "length");
        }
        other => panic!("expected impl, got {other:?}"),
    }

    
    let Item::Function(main) = &prog.items[3] else {
        panic!("expected main function")
    };
    let Stmt::Let(let_stmt) = &main.body.stmts[0] else {
        panic!("expected let")
    };
    assert_eq!(let_stmt.name, "v");
    assert!(matches!(&let_stmt.ty, Some(Type::Named { name, .. }) if name == "Vector2"));
    assert!(matches!(
        &let_stmt.init,
        Some(Expr::Call { callee, .. }) if matches!(
            &**callee,
            Expr::Path { segments, .. } if segments == &["Vector2".to_string(), "new".to_string()]
        )
    ));
}

#[test]
fn parses_generics_match_and_assignments() {
    let src = r#"
fn classify[T](value: T) -> i32 {
    let mut result: i32 = 0;
    match value {
        1 => result = 10,
        2 => { result = 20; }
        _ => result = -1,
    }
    return result;
}
"#;
    let prog = parse(src);
    let Item::Function(f) = &prog.items[0] else {
        panic!("expected function")
    };
    assert_eq!(f.generics, vec!["T".to_string()]);
    assert!(matches!(&f.params[0].ty, Type::Named { name, .. } if name == "T"));

    let Stmt::Expr(Expr::Match { scrutinee, arms, .. }) = &f.body.stmts[1] else {
        panic!("expected match")
    };
    assert!(matches!(&**scrutinee, Expr::Var { name, .. } if name == "value"));
    assert_eq!(arms.len(), 3);
    assert!(matches!(arms[0].pattern, Pattern::Literal(LiteralExpr::Integer(1))));
    assert!(matches!(arms[1].pattern, Pattern::Literal(LiteralExpr::Integer(2))));
    assert!(matches!(arms[1].body, MatchArmBody::Block(_)));
    assert!(matches!(arms[2].pattern, Pattern::Wildcard));
}

#[test]
fn parses_enum_declarations_and_variant_paths() {
    let src = r#"
enum Color { Red, Green, Blue }

fn main() -> i32 {
    let x: i32 = Color::Red;
    return x;
}
"#;
    let prog = parse(src);
    assert_eq!(prog.items.len(), 2);
    let Item::Enum(_) = &prog.items[0] else {
        panic!("expected enum item")
    };
    let Item::Function(main) = &prog.items[1] else {
        panic!("expected main function")
    };
    assert!(matches!(
        &main.body.stmts[0],
        Stmt::Let(let_stmt) if matches!(&let_stmt.init, Some(Expr::Path { segments, .. }) if segments == &["Color".to_string(), "Red".to_string()])
    ));
}

#[test]
fn typechecks_enum_variant_constants() {
    let src = r#"
enum Color { Red, Green, Blue }
fn main() -> i32 {
    let x: i32 = Color::Green;
    return x;
}
"#;
    let errors = check(src);
    assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
}

#[test]
fn parses_enum_class_with_explicit_values() {
    let src = r#"
enum class Color : i32 {
    Red = 10,
    Green = 20,
    Blue = 30
}

fn main() -> i32 {
    return Color::Green;
}
"#;
    let prog = parse(src);
    let Item::Enum(_) = &prog.items[0] else {
        panic!("expected enum class item")
    };
    assert!(matches!(
        &prog.items[1],
        Item::Function(_) if matches!(
            &prog.items[1],
            Item::Function(f) if f.name == "main"
        )
    ));
}

#[test]
fn typechecks_enum_class_with_explicit_values() {
    let src = r#"
enum class Color : i32 {
    Red = 10,
    Green = 20,
    Blue = 30
}
fn main() -> i32 {
    let x: i32 = Color::Green;
    return x;
}
"#;
    let errors = check(src);
    assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
}

#[test]
fn parses_using_type_aliases() {
    let src = r#"
using MyInt = i32;
fn main() -> i32 {
    let x: MyInt = 41 + 1;
    return x;
}
"#;
    let prog = parse(src);
    let Item::TypeAlias(_) = &prog.items[0] else {
        panic!("expected type alias item")
    };
    assert!(matches!(&prog.items[1], Item::Function(_)));
}

#[test]
fn typechecks_using_type_aliases_and_const_declarations() {
    let src = r#"
using MyInt = i32;
const LIMIT: MyInt = 7;
fn main() -> i32 {
    let x: MyInt = LIMIT;
    return x;
}
"#;
    let errors = check(src);
    assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
}

#[test]
fn parses_switch_case_default_and_break_continue() {
    let src = r#"
fn main() -> i32 {
    let value: i32 = 2;
    while true {
        switch value {
            case 1:
                break;
            case 2:
                continue;
            default:
                return 7;
        }
        return 0;
    }
    return 0;
}
"#;
    let prog = parse(src);
    let Item::Function(main) = &prog.items[0] else {
        panic!("expected main function")
    };
    assert!(matches!(main.body.stmts[1], Stmt::While { .. }));
    let Stmt::While(while_stmt) = &main.body.stmts[1] else { panic!("expected while") };
    assert!(matches!(while_stmt.body.stmts[0], Stmt::Switch { .. }));
}

#[test]
fn typechecks_switch_case_default_and_break_continue() {
    let src = r#"
fn main() -> i32 {
    let value: i32 = 2;
    while true {
        switch value {
            case 1:
                break;
            case 2:
                continue;
            default:
                return 7;
        }
        return 0;
    }
    return 0;
}
"#;
    let errors = check(src);
    assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
}

#[test]
fn parser_errors_carry_precise_locations() {
    let err = parse_err("fn f() {\n    let x = ;\n}");
    match err {
        ParseError::UnexpectedToken { span, expected, found } => {
            assert_eq!(span.start.line, 2);
            assert_eq!(span.start.column, 13);
            assert_eq!(expected, "an expression");
            assert_eq!(found, "`;`");
        }
        other => panic!("expected unexpected-token error, got {other:?}"),
    }
}

#[test]
fn parser_reports_unterminated_blocks_at_eof() {
    let err = parse_err("fn f() {\n  while x {\n");
    assert!(matches!(err, ParseError::UnexpectedEof { .. }));
}

#[test]
fn span_covers_multi_line_functions() {
    let prog = parse("fn f() {\n  let a = 1;\n  let b = 2;\n}\n");
    let Item::Function(f) = &prog.items[0] else {
        panic!("expected function")
    };
    assert_eq!(f.span.start.line, 1);
    assert_eq!(f.span.end.line, 4);
    let a = f.body.stmts.first().unwrap();
    assert_eq!(a.span().start.line, 2);
    assert_eq!(a.span().start.column, 3);
}






fn check(src: &str) -> Vec<cprime_compiler::typechecker::TypeError> {
    let prog = parse(src);
    cprime_compiler::typechecker::check_program(&prog)
}

#[test]
fn typechecks_spec_vector2_program() {
    let src = r#"
import std.io;

struct Vector2 {
    x: f64,
    y: f64
}

impl Vector2 {
    fn new(x: f64, y: f64) -> Vector2 {
        return Vector2 { x: x, y: y };
    }

    fn length(self) -> f64 {
        return self.x + self.y;
    }
}

fn main() -> i32 {
    let v: Vector2 = Vector2::new(1.5, 2.5);
    let l: f64 = v.length();
    match 42 {
        42 => println(l),
        _ => return 1,
    }
    return 0;
}
"#;
    let errors = check(src);
    assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
}

#[test]
fn typecheck_errors_report_exact_locations() {
    let src = "fn main() -> i32 {\n    let x: i32 = true;\n    return 0;\n}";
    let errors = check(src);
    assert_eq!(errors.len(), 1, "got: {errors:?}");
    match &errors[0] {
        cprime_compiler::typechecker::TypeError::TypeMismatch { span, .. } => {
            assert_eq!(span.start.line, 2);
            assert_eq!(span.start.column, 18);
        }
        other => panic!("expected type mismatch, got {other:?}"),
    }
}

#[test]
fn typecheck_rejects_unknown_references() {
    let src = r#"
struct Vec { x: f32 }
impl Vec {
    fn sum(self) -> f32 { return self.x; }
}
fn main() {
    let v: Vec = Vec { x: 1.0 };
    let s = v.summ();
    return;
}
"#;
    let errors = check(src);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, cprime_compiler::typechecker::TypeError::UndefinedMethod { .. })),
        "got: {errors:?}"
    );
}

#[test]
fn typecheck_rejects_immutable_assignment() {
    let src = "fn main() {\n    let x: i32 = 1;\n    x = 2;\n    return;\n}";
    let errors = check(src);
    assert!(
        errors
            .iter()
            .any(|e| matches!(
                e,
                cprime_compiler::typechecker::TypeError::AssignToImmutable { .. }
            )),
        "got: {errors:?}"
    );
}

#[test]
fn typechecks_generic_free_functions() {
    let src = r#"
fn swap[T](a: *mut T, b: *mut T) -> T {
    let t: T = *a;
    *a = *b;
    *b = t;
    return *a;
}

fn identity[T](x: T) -> T {
    return x;
}

fn main() -> i32 {
    let x: i32 = 1;
    let y: i32 = 2;
    let s: i32 = swap(&mut x, &mut y);
    let z: f64 = identity(3.5);
    return s;
}
"#;
    let errors = check(src);
    assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
}

#[test]
fn typechecks_generic_structs_methods_and_impl_generics() {
    let src = r#"
struct Opt[T] { value: T }

impl Opt[T] {
    fn new(v: T) -> Opt[T] {
        return Opt { value: v };
    }
    fn get(self) -> T {
        return self.value;
    }
    fn into_opt_pair(self, other: T) -> Pair[T, T] {
        return Pair { first: self.value, second: other };
    }
}

struct Pair[A, B] {
    first: A,
    second: B
}

fn main() -> i32 {
    let o: Opt[i32] = Opt { value: 7 };
    let v: i32 = o.get();
    let p: Opt[f64] = Opt::new(1.5);
    let pair: Pair[i32, i32] = o.into_opt_pair(9);
    return pair.first + pair.second;
}
"#;
    let errors = check(src);
    assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
}

#[test]
fn typecheck_rejects_impl_generic_mismatch() {
    let src = r#"
struct Opt[T] { value: T }
impl Opt[A, B] {
    fn get(self) -> T { return self.value; }
}
fn main() { return; }
"#;
    let errors = check(src);
    assert!(
        errors
            .iter()
            .any(|e| matches!(e, cprime_compiler::typechecker::TypeError::ImplGenericMismatch { .. })),
        "got: {errors:?}"
    );
}
