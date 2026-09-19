







use std::fmt;
use std::sync::Arc;






#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceLocation {
    
    pub file: Arc<str>,
    
    pub line: u32,
    
    pub column: u32,
    
    pub offset: usize,
}

impl SourceLocation {
    
    pub fn new(file: impl Into<Arc<str>>, line: u32, column: u32, offset: usize) -> Self {
        SourceLocation {
            file: file.into(),
            line,
            column,
            offset,
        }
    }
}

impl fmt::Display for SourceLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}


#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SourceSpan {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl SourceSpan {
    
    pub fn new(start: SourceLocation, end: SourceLocation) -> Self {
        SourceSpan { start, end }
    }

    
    pub fn location(&self) -> &SourceLocation {
        &self.start
    }
}

impl fmt::Display for SourceSpan {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.start.line == self.end.line {
            write!(
                f,
                "{}:{}:{}",
                self.start.file, self.start.line, self.start.column
            )
        } else {
            write!(
                f,
                "{}:{}:{}-{}:{}",
                self.start.file, self.start.line, self.start.column, self.end.line, self.end.column
            )
        }
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    Let,
    Mut,
    Fn,
    Return,
    If,
    Else,
    While,
    For,
    In,
    Match,
    Switch,
    Case,
    Default,
    Break,
    Continue,
    Struct,
    Class,
    Impl,
    Namespace,
    Import,
    Use,
    Using,
    As,
    Pub,
    Private,
    Protected,
    Const,
    Enum,
    Virtual,
    Override,
    Final,
    Abstract,
    Static,
    Extern,
    True,
    False,
    Null,
    SelfKw,
    SelfType,
    
    Void,
    Char,
    Bool,
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

impl Keyword {
    
    
    
    
    
    
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(name: &str) -> Option<Keyword> {
        use Keyword::*;
        Some(match name {
            "let" => Let,
            "mut" => Mut,
            "fn" => Fn,
            "return" => Return,
            "if" => If,
            "else" => Else,
            "while" => While,
            "for" => For,
            "in" => In,
            "match" => Match,
            "switch" => Switch,
            "case" => Case,
            "default" => Default,
            "break" => Break,
            "continue" => Continue,
            "struct" => Struct,
            "class" => Class,
            "impl" => Impl,
            "namespace" => Namespace,
            "import" => Import,
            "use" => Use,
            "using" => Using,
            "as" => As,
            "pub" => Pub,
            "private" => Private,
            "protected" => Protected,
            "const" => Const,
            "enum" => Enum,
            "virtual" => Virtual,
            "override" => Override,
            "final" => Final,
            "abstract" => Abstract,
            "static" => Static,
            "extern" => Extern,
            "true" => True,
            "false" => False,
            "null" => Null,
            "self" => SelfKw,
            "Self" => SelfType,
            "void" => Void,
            "char" => Char,
            "bool" => Bool,
            "i8" => I8,
            "i16" => I16,
            "i32" => I32,
            "i64" => I64,
            "i128" => I128,
            "u8" => U8,
            "u16" => U16,
            "u32" => U32,
            "u64" => U64,
            "u128" => U128,
            "f32" => F32,
            "f64" => F64,
            "isize" => Isize,
            "usize" => Usize,
            _ => return None,
        })
    }

    
    pub fn as_str(self) -> &'static str {
        use Keyword::*;
        match self {
            Let => "let",
            Mut => "mut",
            Fn => "fn",
            Return => "return",
            If => "if",
            Else => "else",
            While => "while",
            For => "for",
            In => "in",
            Match => "match",
            Switch => "switch",
            Case => "case",
            Default => "default",
            Break => "break",
            Continue => "continue",
            Struct => "struct",
            Class => "class",
            Impl => "impl",
            Namespace => "namespace",
            Import => "import",
            Use => "use",
            Using => "using",
            As => "as",
            Pub => "pub",
            Private => "private",
            Protected => "protected",
            Const => "const",
            Enum => "enum",
            Virtual => "virtual",
            Override => "override",
            Final => "final",
            Abstract => "abstract",
            Static => "static",
            Extern => "extern",
            True => "true",
            False => "false",
            Null => "null",
            SelfKw => "self",
            SelfType => "Self",
            Void => "void",
            Char => "char",
            Bool => "bool",
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

impl fmt::Display for Keyword {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Symbol {
    
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Semicolon,
    Colon,
    Comma,
    Dot,

    
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    
    Eq,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,

    
    EqEq,
    Bang,
    BangEq,
    Lt,
    Gt,
    LtEq,
    GtEq,

    
    Amp,
    Pipe,
    Caret,
    Tilde,
    Shl,
    Shr,

    
    AndAnd,
    OrOr,

    
    Arrow,     
    FatArrow,  
    PathSep,   
    Question,  
    Underscore, 
    DotDot,    
    DotDotEq,  
    Ellipsis,  
}

impl Symbol {
    
    
    
    
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Symbol> {
        use Symbol::*;
        Some(match s {
            "(" => LeftParen,
            ")" => RightParen,
            "{" => LeftBrace,
            "}" => RightBrace,
            "[" => LeftBracket,
            "]" => RightBracket,
            ";" => Semicolon,
            ":" => Colon,
            "," => Comma,
            "." => Dot,
            "+" => Plus,
            "-" => Minus,
            "*" => Star,
            "/" => Slash,
            "%" => Percent,
            "=" => Eq,
            "+=" => PlusEq,
            "-=" => MinusEq,
            "*=" => StarEq,
            "/=" => SlashEq,
            "%=" => PercentEq,
            "==" => EqEq,
            "!" => Bang,
            "!=" => BangEq,
            "<" => Lt,
            ">" => Gt,
            "<=" => LtEq,
            ">=" => GtEq,
            "&" => Amp,
            "|" => Pipe,
            "^" => Caret,
            "~" => Tilde,
            "<<" => Shl,
            ">>" => Shr,
            "&&" => AndAnd,
            "||" => OrOr,
            "->" => Arrow,
            "=>" => FatArrow,
            "::" => PathSep,
            "?" => Question,
            "_" => Underscore,
            ".." => DotDot,
            "..=" => DotDotEq,
            "..." => Ellipsis,
            _ => return None,
        })
    }

    
    pub fn as_str(self) -> &'static str {
        use Symbol::*;
        match self {
            LeftParen => "(",
            RightParen => ")",
            LeftBrace => "{",
            RightBrace => "}",
            LeftBracket => "[",
            RightBracket => "]",
            Semicolon => ";",
            Colon => ":",
            Comma => ",",
            Dot => ".",
            Plus => "+",
            Minus => "-",
            Star => "*",
            Slash => "/",
            Percent => "%",
            Eq => "=",
            PlusEq => "+=",
            MinusEq => "-=",
            StarEq => "*=",
            SlashEq => "/=",
            PercentEq => "%=",
            EqEq => "==",
            Bang => "!",
            BangEq => "!=",
            Lt => "<",
            Gt => ">",
            LtEq => "<=",
            GtEq => ">=",
            Amp => "&",
            Pipe => "|",
            Caret => "^",
            Tilde => "~",
            Shl => "<<",
            Shr => ">>",
            AndAnd => "&&",
            OrOr => "||",
            Arrow => "->",
            FatArrow => "=>",
            PathSep => "::",
            Question => "?",
            Underscore => "_",
            DotDot => "..",
            DotDotEq => "..=",
            Ellipsis => "...",
        }
    }
}

impl fmt::Display for Symbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Radix {
    Decimal,
    Hex,
    Octal,
    Binary,
}

impl Radix {
    pub fn base(self) -> u32 {
        match self {
            Radix::Decimal => 10,
            Radix::Hex => 16,
            Radix::Octal => 8,
            Radix::Binary => 2,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Radix::Decimal => "decimal",
            Radix::Hex => "hexadecimal",
            Radix::Octal => "octal",
            Radix::Binary => "binary",
        }
    }
}

impl fmt::Display for Radix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}



#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntegerLiteral {
    pub value: i128,
    pub radix: Radix,
}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FloatLiteral {
    pub value: f64,
}


#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    
    Identifier(String),
    
    Integer(IntegerLiteral),
    
    Float(FloatLiteral),
    
    Char(char),
    
    String(String),
    
    Keyword(Keyword),
    
    Symbol(Symbol),
    
    Eof,
}

impl TokenKind {
    
    pub fn is_eof(&self) -> bool {
        matches!(self, TokenKind::Eof)
    }

    
    pub fn describe(&self) -> String {
        match self {
            TokenKind::Identifier(name) => format!("identifier `{name}`"),
            TokenKind::Integer(lit) => format!("integer literal `{lit}`"),
            TokenKind::Float(lit) => format!("float literal `{lit}`"),
            TokenKind::Char(c) => format!("character literal `{c}`"),
            TokenKind::String(s) => format!("string literal `{s}`"),
            TokenKind::Keyword(k) => format!("keyword `{k}`"),
            TokenKind::Symbol(s) => format!("`{s}`"),
            TokenKind::Eof => "end of file".to_string(),
        }
    }
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Identifier(name) => f.write_str(name),
            TokenKind::Integer(lit) => write!(f, "{lit}"),
            TokenKind::Float(lit) => write!(f, "{lit}"),
            TokenKind::Char(c) => write!(f, "{c}"),
            TokenKind::String(s) => f.write_str(s),
            TokenKind::Keyword(k) => f.write_str(k.as_str()),
            TokenKind::Symbol(s) => f.write_str(s.as_str()),
            TokenKind::Eof => f.write_str("<eof>"),
        }
    }
}

impl fmt::Display for IntegerLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl fmt::Display for FloatLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}





#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum LexError {
    #[error("{location}: invalid character `{c}`")]
    InvalidCharacter { location: SourceLocation, c: char },

    #[error("{location}: unterminated string literal")]
    UnterminatedString { location: SourceLocation },

    #[error("{location}: unterminated character literal")]
    UnterminatedChar { location: SourceLocation },

    #[error("{location}: unterminated block comment")]
    UnterminatedBlockComment { location: SourceLocation },

    #[error("{location}: empty character literal")]
    EmptyCharLiteral { location: SourceLocation },

    #[error("{location}: character literal may not contain more than one character")]
    MultiCharLiteral { location: SourceLocation },

    #[error("{location}: invalid escape sequence `\\{escape}`")]
    InvalidEscape { location: SourceLocation, escape: String },

    #[error("{location}: `\\x` escape requires 1 or 2 hex digits")]
    InvalidHexEscape { location: SourceLocation },

    #[error("{location}: invalid `\\u{{...}}` escape")]
    InvalidUnicodeEscape { location: SourceLocation },

    #[error("{location}: invalid {radix} literal")]
    InvalidRadixLiteral { location: SourceLocation, radix: Radix },

    #[error("{location}: integer literal is too large")]
    IntegerOverflow { location: SourceLocation },

    #[error("{location}: invalid numeric literal")]
    InvalidNumber { location: SourceLocation },
}


#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    
    pub span: SourceSpan,
}

impl Token {
    
    pub fn new(kind: TokenKind, span: SourceSpan) -> Token {
        Token { kind, span }
    }

    
    pub fn location(&self) -> &SourceLocation {
        &self.span.start
    }

    
    pub fn is_eof(&self) -> bool {
        self.kind.is_eof()
    }
}
