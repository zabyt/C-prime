





use std::fmt;

use crate::token::{Keyword, SourceSpan};


#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub items: Vec<Item>,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Import(ImportDecl),
    Function(Function),
    Struct(StructDecl),
    Enum(EnumDecl),
    TypeAlias(TypeAliasDecl),
    Const(ConstDecl),
    Impl(ImplBlock),
    Namespace(NamespaceDecl),
    
    ExternFunction(ExternFunctionDecl),
}

impl Item {
    
    pub fn span(&self) -> &SourceSpan {
        match self {
            Item::Import(d) => &d.span,
            Item::Function(f) => &f.span,
            Item::Struct(s) => &s.span,
            Item::Enum(e) => &e.span,
            Item::TypeAlias(a) => &a.span,
            Item::Const(c) => &c.span,
            Item::Impl(b) => &b.span,
            Item::Namespace(n) => &n.span,
            Item::ExternFunction(e) => &e.span,
        }
    }

    
    
    pub fn summary(&self) -> String {
        match self {
            Item::Import(d) => format!("import {}", d.path.join(".")),
            Item::Function(f) => format!("fn {}({} param{})", f.name, f.params.len(), if f.params.len() == 1 { "" } else { "s" }),
            Item::Struct(s) => format!("struct {}", s.name),
            Item::Enum(e) => format!("enum {}", e.name),
            Item::TypeAlias(a) => format!("using {} = {}", a.name, a.target),
            Item::Const(c) => format!("const {}: {}", c.name, c.ty),
            Item::Impl(b) => format!("impl {} ({} method{})", b.type_name, b.methods.len(), if b.methods.len() == 1 { "" } else { "s" }),
            Item::Namespace(n) => format!("namespace {}", n.name),
            Item::ExternFunction(e) => format!("extern fn {} ({} param{})", e.name, e.params.len(), if e.params.len() == 1 { "" } else { "s" }),
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct TypeAliasDecl {
    pub name: String,
    pub target: Type,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct ConstDecl {
    pub name: String,
    pub ty: Type,
    pub value: Expr,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub value: Option<Expr>,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct EnumDecl {
    pub name: String,
    pub base: Option<Type>,
    pub variants: Vec<EnumVariant>,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct NamespaceDecl {
    pub name: String,
    pub items: Vec<Item>,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    
    pub path: Vec<String>,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
}


#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    
    pub generics: Vec<String>,
    pub params: Vec<Param>,
    pub visibility: Visibility,
    pub is_virtual: bool,
    pub is_override: bool,
    pub is_final: bool,
    pub is_abstract: bool,
    pub is_static: bool,
    
    pub is_const: bool,
    
    pub return_ty: Option<Type>,
    pub body: Block,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub span: SourceSpan,
}




#[derive(Debug, Clone, PartialEq)]
pub struct ExternFunctionDecl {
    pub name: String,
    pub params: Vec<Param>,
    
    pub return_ty: Option<Type>,
    
    
    
    pub is_variadic: bool,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct StructDecl {
    pub name: String,
    pub generics: Vec<String>,
    pub base: Option<Type>,
    pub is_final: bool,
    pub is_abstract: bool,
    pub fields: Vec<FieldDecl>,
    pub methods: Vec<Function>,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct FieldDecl {
    pub name: String,
    pub visibility: Visibility,
    pub ty: Type,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct ImplBlock {
    
    pub type_name: String,
    
    
    pub generics: Vec<String>,
    pub methods: Vec<Function>,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let(LetStmt),
    
    Expr(Expr),
    If(IfStmt),
    While(WhileStmt),
    For(ForStmt),
    Switch(SwitchStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    
    Block(Block),
}

impl Stmt {
    
    pub fn span(&self) -> &SourceSpan {
        match self {
            Stmt::Let(s) => &s.span,
            Stmt::Expr(e) => e.span(),
            Stmt::If(s) => &s.span,
            Stmt::While(s) => &s.span,
            Stmt::For(s) => &s.span,
            Stmt::Switch(s) => &s.span,
            Stmt::Break(s) => &s.span,
            Stmt::Continue(s) => &s.span,
            Stmt::Block(b) => &b.span,
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct SwitchStmt {
    pub value: Expr,
    pub arms: Vec<SwitchArm>,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct SwitchArm {
    pub pattern: Option<Expr>,
    pub body: Vec<Stmt>,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct BreakStmt {
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct ContinueStmt {
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct LetStmt {
    pub name: String,
    pub mutable: bool,
    pub ty: Option<Type>,
    pub init: Option<Expr>,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub condition: Expr,
    pub then_block: Block,
    pub else_branch: Option<ElseBranch>,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub enum ElseBranch {
    
    If(Box<IfStmt>),
    
    Block(Block),
}


#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Block,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    
    pub variable: String,
    pub iterable: Expr,
    pub body: Block,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Primitive(PrimitiveType),
    
    
    Named { name: String, args: Vec<Type>, span: SourceSpan },
    
    Pointer { pointee: Box<Type>, mutability: PointerMutability, span: SourceSpan },
    
    Array { element: Box<Type>, length: Box<Expr>, span: SourceSpan },
}

impl Type {
    
    pub fn span(&self) -> &SourceSpan {
        match self {
            Type::Primitive(_) => unreachable!("primitive types carry no span; use the caller's span"),
            Type::Named { span, .. } => span,
            Type::Pointer { span, .. } => span,
            Type::Array { span, .. } => span,
        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointerMutability {
    Const,
    Mut,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    Void,
    Bool,
    Char,
    I8,
    I16,
    I32,
    I64,
    I128,
    U8,
    U16,
    U32,
    U64,
    U128,
    F32,
    F64,
    Isize,
    Usize,
}

impl PrimitiveType {
    
    
    pub fn from_keyword(kw: Keyword) -> Option<PrimitiveType> {
        use Keyword::*;
        Some(match kw {
            Void => PrimitiveType::Void,
            Bool => PrimitiveType::Bool,
            Char => PrimitiveType::Char,
            I8 => PrimitiveType::I8,
            I16 => PrimitiveType::I16,
            I32 => PrimitiveType::I32,
            I64 => PrimitiveType::I64,
            I128 => PrimitiveType::I128,
            U8 => PrimitiveType::U8,
            U16 => PrimitiveType::U16,
            U32 => PrimitiveType::U32,
            U64 => PrimitiveType::U64,
            U128 => PrimitiveType::U128,
            F32 => PrimitiveType::F32,
            F64 => PrimitiveType::F64,
            Isize => PrimitiveType::Isize,
            Usize => PrimitiveType::Usize,
            _ => return None,
        })
    }

    
    pub fn as_str(self) -> &'static str {
        use PrimitiveType::*;
        match self {
            Void => "void",
            Bool => "bool",
            Char => "char",
            I8 => "i8",
            I16 => "i16",
            I32 => "i32",
            I64 => "i64",
            I128 => "i128",
            U8 => "u8",
            U16 => "u16",
            U32 => "u32",
            U64 => "u64",
            U128 => "u128",
            F32 => "f32",
            F64 => "f64",
            Isize => "isize",
            Usize => "usize",
        }
    }
}

impl fmt::Display for PrimitiveType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Primitive(p) => p.fmt(f),
            Type::Named { name, args, .. } => {
                f.write_str(name)?;
                if !args.is_empty() {
                    write!(f, "[")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        arg.fmt(f)?;
                    }
                    write!(f, "]")?;
                }
                Ok(())
            }
            Type::Pointer { pointee, mutability, .. } => match mutability {
                PointerMutability::Const => write!(f, "*const {}", pointee),
                PointerMutability::Mut => write!(f, "*mut {}", pointee),
            },
            Type::Array { element, length, .. } => write!(f, "[{}; {:?}]", element, length),
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(LiteralExpr, SourceSpan),
    
    Var { name: String, span: SourceSpan },
    
    Path { segments: Vec<String>, span: SourceSpan },
    
    Call { callee: Box<Expr>, args: Vec<Expr>, span: SourceSpan },
    
    MethodCall { receiver: Box<Expr>, method: String, args: Vec<Expr>, span: SourceSpan },
    
    FieldAccess { base: Box<Expr>, field: String, span: SourceSpan },
    
    Index { base: Box<Expr>, index: Box<Expr>, span: SourceSpan },
    
    Unary { op: UnaryOp, operand: Box<Expr>, span: SourceSpan },
    
    Binary { op: BinaryOp, lhs: Box<Expr>, rhs: Box<Expr>, span: SourceSpan },
    
    Assign { target: Box<Expr>, op: AssignOp, value: Box<Expr>, span: SourceSpan },
    
    Cast { expr: Box<Expr>, ty: Box<Type>, span: SourceSpan },
    
    
    StructLiteral { name: String, args: Vec<Type>, fields: Vec<(String, Expr)>, span: SourceSpan },
    
    Match { scrutinee: Box<Expr>, arms: Vec<MatchArm>, span: SourceSpan },
    
    Return { value: Option<Box<Expr>>, span: SourceSpan },
    
    Range { start: Box<Expr>, end: Box<Expr>, inclusive: bool, span: SourceSpan },
    
    ArrayRepeat { value: Box<Expr>, count: Box<Expr>, span: SourceSpan },
    
    ArrayLiteral { elements: Vec<Expr>, span: SourceSpan },
    
    SizeOf { ty: Box<Type>, span: SourceSpan },
}

impl Expr {
    
    pub fn span(&self) -> &SourceSpan {
        match self {
            Expr::Literal(_, span) => span,
            Expr::Var { span, .. } => span,
            Expr::Path { span, .. } => span,
            Expr::Call { span, .. } => span,
            Expr::MethodCall { span, .. } => span,
            Expr::FieldAccess { span, .. } => span,
            Expr::Index { span, .. } => span,
            Expr::Unary { span, .. } => span,
            Expr::Binary { span, .. } => span,
            Expr::Assign { span, .. } => span,
            Expr::Cast { span, .. } => span,
            Expr::StructLiteral { span, .. } => span,
            Expr::Match { span, .. } => span,
            Expr::Return { span, .. } => span,
            Expr::Range { span, .. } => span,
            Expr::ArrayRepeat { span, .. } => span,
            Expr::ArrayLiteral { span, .. } => span,
            Expr::SizeOf { span, .. } => span,
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub enum LiteralExpr {
    Integer(i128),
    Float(f64),
    Char(char),
    String(String),
    Bool(bool),
    
    Null,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
    Deref,
    AddrOf,
    AddrOfMut,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    AndAnd,
    OrOr,
    BitAnd,
    BitOr,
    BitXor,
    Shl,
    Shr,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}


#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: MatchArmBody,
    pub span: SourceSpan,
}


#[derive(Debug, Clone, PartialEq)]
pub enum MatchArmBody {
    Expr(Expr),
    Block(Block),
}


#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    
    Wildcard,
    
    Literal(LiteralExpr),
    
    Binding(String),
}
