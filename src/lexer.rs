









use std::iter::Peekable;
use std::str::CharIndices;
use std::sync::Arc;

use crate::token::{
    FloatLiteral, IntegerLiteral, Keyword, LexError, Radix, SourceLocation, SourceSpan, Symbol,
    Token, TokenKind,
};


pub type LexResult<T> = Result<T, LexError>;



#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EscapeContext {
    StringLiteral,
    CharLiteral,
}






fn is_ident_start(c: char) -> bool {
    c == '_' || c.is_alphabetic()
}


fn is_ident_continue(c: char) -> bool {
    c == '_' || c.is_alphanumeric()
}







pub struct Lexer<'a> {
    src: &'a str,
    chars: Peekable<CharIndices<'a>>,
    file: Arc<str>,
    line: u32,
    column: u32,
    offset: usize,
    done: bool,
}

impl<'a> Lexer<'a> {
    
    
    pub fn new(source: &'a str, file_name: impl Into<Arc<str>>) -> Self {
        Lexer {
            src: source,
            chars: source.char_indices().peekable(),
            file: file_name.into(),
            line: 1,
            column: 1,
            offset: 0,
            done: false,
        }
    }

    
    pub fn source(&self) -> &'a str {
        self.src
    }

    
    pub fn location(&self) -> SourceLocation {
        SourceLocation::new(self.file.clone(), self.line, self.column, self.offset)
    }

    
    fn peek(&mut self) -> Option<char> {
        self.chars.peek().map(|&(_, c)| c)
    }

    
    fn peek2(&mut self) -> Option<char> {
        let mut it = self.chars.clone();
        it.next();
        it.peek().map(|&(_, c)| c)
    }

    
    fn peek3(&mut self) -> Option<char> {
        let mut it = self.chars.clone();
        it.next();
        it.next();
        it.peek().map(|&(_, c)| c)
    }

    
    
    
    
    
    fn bump(&mut self) -> Option<char> {
        let (idx, c) = self.chars.next()?;
        self.offset = idx + c.len_utf8();
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    
    
    
    
    fn next_token(&mut self) -> LexResult<Token> {
        self.skip_trivia()?;
        let start = self.location();
        let c = match self.peek() {
            Some(c) => c,
            None => {
                let end = self.location();
                return Ok(Token::new(TokenKind::Eof, SourceSpan::new(start, end)));
            }
        };

        let kind = if is_ident_start(c) {
            self.ident_or_keyword()
        } else if c.is_ascii_digit() || (c == '.' && self.peek2().is_some_and(|d| d.is_ascii_digit()))
        {
            self.number(start.clone())?
        } else if c == '\'' {
            self.char_literal(start.clone())?
        } else if c == '"' {
            self.string_literal(start.clone())?
        } else {
            self.symbol()?
        };

        let end = self.location();
        Ok(Token::new(kind, SourceSpan::new(start, end)))
    }

    
    fn skip_trivia(&mut self) -> LexResult<()> {
        loop {
            match self.peek() {
                Some(' ') | Some('\t') | Some('\x0b') | Some('\x0c') | Some('\u{feff}') => {
                    self.bump();
                }
                Some('\r') => {
                    self.bump();
                    if self.peek() == Some('\n') {
                        self.bump();
                    } else {
                        
                        self.line += 1;
                        self.column = 1;
                    }
                }
                Some('\n') => {
                    self.bump();
                }
                Some('/') if self.peek2() == Some('/') => {
                    self.skip_line_comment();
                }
                Some('/') if self.peek2() == Some('*') => {
                    self.skip_block_comment()?;
                }
                _ => return Ok(()),
            }
        }
    }

    
    fn skip_line_comment(&mut self) {
        while let Some(c) = self.peek() {
            if c == '\n' || c == '\r' {
                break;
            }
            self.bump();
        }
    }

    
    fn skip_block_comment(&mut self) -> LexResult<()> {
        let start = self.location();
        self.bump(); 
        self.bump(); 
        let mut depth: u32 = 1;
        while depth > 0 {
            match (self.peek(), self.peek2()) {
                (Some('/'), Some('*')) => {
                    depth += 1;
                    self.bump();
                    self.bump();
                }
                (Some('*'), Some('/')) => {
                    depth -= 1;
                    self.bump();
                    self.bump();
                }
                (Some(_), _) => {
                    self.bump();
                }
                (None, _) => {
                    return Err(LexError::UnterminatedBlockComment { location: start });
                }
            }
        }
        Ok(())
    }

    
    
    fn ident_or_keyword(&mut self) -> TokenKind {
        let mut name = String::new();
        while let Some(c) = self.peek() {
            if is_ident_continue(c) {
                name.push(c);
                self.bump();
            } else {
                break;
            }
        }
        if name == "_" {
            return TokenKind::Symbol(Symbol::Underscore);
        }
        match Keyword::from_str(&name) {
            Some(kw) => TokenKind::Keyword(kw),
            None => TokenKind::Identifier(name),
        }
    }

    
    
    fn number(&mut self, start: SourceLocation) -> LexResult<TokenKind> {
        if self.peek() == Some('0') {
            match self.peek2() {
                Some('x') | Some('X') => {
                    self.bump();
                    self.bump();
                    return self.radix_integer(Radix::Hex, start);
                }
                Some('o') | Some('O') => {
                    self.bump();
                    self.bump();
                    return self.radix_integer(Radix::Octal, start);
                }
                Some('b') | Some('B') => {
                    self.bump();
                    self.bump();
                    return self.radix_integer(Radix::Binary, start);
                }
                _ => {}
            }
        }
        self.decimal_number(start)
    }

    
    fn radix_integer(&mut self, radix: Radix, start: SourceLocation) -> LexResult<TokenKind> {
        let base = radix.base() as u128;
        let mut value: u128 = 0;
        let mut any_digit = false;
        while let Some(c) = self.peek() {
            let Some(d) = c.to_digit(base as u32) else {
                break;
            };
            let d = d as u128;
            value = value
                .checked_mul(base)
                .and_then(|v| v.checked_add(d))
                .ok_or(LexError::IntegerOverflow {
                    location: start.clone(),
                })?;
            any_digit = true;
            self.bump();
        }
        if !any_digit {
            return Err(LexError::InvalidRadixLiteral {
                location: start,
                radix,
            });
        }
        if value > i128::MAX as u128 {
            return Err(LexError::IntegerOverflow { location: start });
        }
        Ok(TokenKind::Integer(IntegerLiteral {
            value: value as i128,
            radix,
        }))
    }

    
    
    
    
    
    fn decimal_number(&mut self, start: SourceLocation) -> LexResult<TokenKind> {
        let mut int_digits = String::new();
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                int_digits.push(c);
                self.bump();
            } else {
                break;
            }
        }

        let mut frac_digits = String::new();
        let is_dot = self.peek() == Some('.') && self.peek2() != Some('.');
        if is_dot {
            self.bump();
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    frac_digits.push(c);
                    self.bump();
                } else {
                    break;
                }
            }
        }

        let mut exp_digits = String::new();
        let is_exp = matches!(self.peek(), Some('e') | Some('E'))
            && (matches!(self.peek2(), Some(d) if d.is_ascii_digit())
                || (matches!(self.peek2(), Some('+') | Some('-'))
                    && matches!(self.peek3(), Some(d) if d.is_ascii_digit())));
        if is_exp {
            self.bump();
            if matches!(self.peek(), Some('+') | Some('-')) {
                exp_digits.push(self.peek().unwrap());
                self.bump();
            }
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    exp_digits.push(c);
                    self.bump();
                } else {
                    break;
                }
            }
        }

        if is_dot || is_exp {
            let mut repr = int_digits;
            if !frac_digits.is_empty() {
                repr.push('.');
                repr.push_str(&frac_digits);
            }
            if is_exp {
                repr.push('e');
                repr.push_str(&exp_digits);
            }
            let value = repr
                .parse::<f64>()
                .map_err(|_| LexError::InvalidNumber {
                    location: start.clone(),
                })?;
            return Ok(TokenKind::Float(FloatLiteral { value }));
        }

        if int_digits.is_empty() {
            return Err(LexError::InvalidNumber { location: start });
        }
        let value = int_digits
            .parse::<u128>()
            .map_err(|_| LexError::IntegerOverflow {
                location: start.clone(),
            })?;
        if value > i128::MAX as u128 {
            return Err(LexError::IntegerOverflow { location: start });
        }
        Ok(TokenKind::Integer(IntegerLiteral {
            value: value as i128,
            radix: Radix::Decimal,
        }))
    }

    
    fn char_literal(&mut self, start: SourceLocation) -> LexResult<TokenKind> {
        self.bump(); 
        let c = match self.peek() {
            None | Some('\n') => {
                return Err(LexError::UnterminatedChar { location: start })
            }
            Some('\'') => {
                return Err(LexError::EmptyCharLiteral { location: start })
            }
            Some('\\') => self.escape(EscapeContext::CharLiteral, &start)?,
            Some(ch) => {
                self.bump();
                ch
            }
        };
        if self.peek() != Some('\'') {
            return match self.peek() {
                
                
                None | Some('\n') => Err(LexError::UnterminatedChar { location: start }),
                
                
                Some(_) => Err(LexError::MultiCharLiteral { location: start }),
            };
        }
        self.bump(); 
        Ok(TokenKind::Char(c))
    }

    
    fn string_literal(&mut self, start: SourceLocation) -> LexResult<TokenKind> {
        self.bump(); 
        let mut value = String::new();
        loop {
            match self.peek() {
                None | Some('\n') => {
                    return Err(LexError::UnterminatedString { location: start })
                }
                Some('"') => {
                    self.bump();
                    break;
                }
                Some('\\') => value.push(self.escape(EscapeContext::StringLiteral, &start)?),
                Some(ch) => {
                    value.push(ch);
                    self.bump();
                }
            }
        }
        Ok(TokenKind::String(value))
    }

    
    
    fn escape(&mut self, ctx: EscapeContext, start: &SourceLocation) -> LexResult<char> {
        self.bump(); 
        let c = match self.peek() {
            None => {
                let location = start.clone();
                return Err(match ctx {
                    EscapeContext::StringLiteral => LexError::UnterminatedString { location },
                    EscapeContext::CharLiteral => LexError::UnterminatedChar { location },
                });
            }
            Some(c) => c,
        };

        let escaped = match c {
            'n' => '\n',
            't' => '\t',
            'r' => '\r',
            '0' => '\0',
            '\\' => '\\',
            '\'' => '\'',
            '"' => '"',
            'a' => '\x07',
            'b' => '\x08',
            'f' => '\x0c',
            'v' => '\x0b',
            'e' => '\x1b',
            'x' => return self.hex_escape(start),
            'u' => return self.unicode_escape(start),
            other => {
                return Err(LexError::InvalidEscape {
                    location: self.location(),
                    escape: other.to_string(),
                })
            }
        };
        self.bump();
        Ok(escaped)
    }

    
    fn hex_escape(&mut self, start: &SourceLocation) -> LexResult<char> {
        self.bump(); 
        let mut value: u32 = 0;
        let mut count: u32 = 0;
        while count < 2 {
            match self.peek().and_then(|c| c.to_digit(16)) {
                Some(d) => {
                    value = value * 16 + d;
                    count += 1;
                    self.bump();
                }
                None => break,
            }
        }
        if count == 0 {
            return Err(LexError::InvalidHexEscape {
                location: start.clone(),
            });
        }
        Ok(char::from_u32(value).unwrap_or('\u{fffd}'))
    }

    
    fn unicode_escape(&mut self, start: &SourceLocation) -> LexResult<char> {
        self.bump(); 
        if self.peek() != Some('{') {
            return Err(LexError::InvalidUnicodeEscape {
                location: start.clone(),
            });
        }
        self.bump(); 
        let mut value: u32 = 0;
        let mut count: u32 = 0;
        loop {
            match self.peek() {
                Some('}') => {
                    self.bump();
                    break;
                }
                Some(c) => match c.to_digit(16) {
                    Some(d) if count < 6 => {
                        value = value * 16 + d;
                        count += 1;
                        self.bump();
                    }
                    _ => {
                        return Err(LexError::InvalidUnicodeEscape {
                            location: start.clone(),
                        })
                    }
                },
                None => {
                    return Err(LexError::InvalidUnicodeEscape {
                        location: start.clone(),
                    })
                }
            }
        }
        if count == 0 {
            return Err(LexError::InvalidUnicodeEscape {
                location: start.clone(),
            });
        }
        char::from_u32(value).ok_or(LexError::InvalidUnicodeEscape {
            location: start.clone(),
        })
    }

    
    
    fn symbol(&mut self) -> LexResult<TokenKind> {
        let location = self.location();
        let (c1, c2) = match (self.peek(), self.peek2()) {
            (Some(c1), Some(c2)) => (c1, c2),
            (Some(c1), None) => {
                let single = c1.to_string();
                match Symbol::from_str(&single) {
                    Some(sym) => {
                        self.bump();
                        return Ok(TokenKind::Symbol(sym));
                    }
                    None => return Err(LexError::InvalidCharacter { location, c: c1 }),
                }
            }
            (None, _) => unreachable!("symbol() called at end of input"),
        };

        let pair = format!("{c1}{c2}");

        
        if let Some(c3) = self.peek3() {
            let triple = format!("{c1}{c2}{c3}");
            if let Some(sym) = Symbol::from_str(&triple) {
                self.bump();
                self.bump();
                self.bump();
                return Ok(TokenKind::Symbol(sym));
            }
        }

        if let Some(sym) = Symbol::from_str(&pair) {
            self.bump();
            self.bump();
            return Ok(TokenKind::Symbol(sym));
        }

        let single = c1.to_string();
        match Symbol::from_str(&single) {
            Some(sym) => {
                self.bump();
                Ok(TokenKind::Symbol(sym))
            }
            None => Err(LexError::InvalidCharacter { location, c: c1 }),
        }
    }

    
    
    pub fn tokenize(mut self) -> LexResult<Vec<Token>> {
        let mut tokens = Vec::new();
        for token in self.by_ref() {
            let token = token?;
            let eof = token.is_eof();
            tokens.push(token);
            if eof {
                break;
            }
        }
        Ok(tokens)
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = LexResult<Token>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        match self.next_token() {
            Ok(token) => {
                let eof = token.is_eof();
                if eof {
                    self.done = true;
                }
                Some(Ok(token))
            }
            Err(err) => {
                self.done = true;
                Some(Err(err))
            }
        }
    }
}


pub fn lex(source: &str, file_name: impl Into<Arc<str>>) -> LexResult<Vec<Token>> {
    Lexer::new(source, file_name).tokenize()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::Keyword::*;
    use crate::token::{Keyword, Radix, Symbol};

    
    fn kinds(src: &str) -> Vec<TokenKind> {
        lex(src, "test.cp")
            .unwrap_or_else(|e| panic!("unexpected lex error: {e}"))
            .into_iter()
            .map(|t| t.kind)
            .collect()
    }

    
    fn error_of(src: &str) -> LexError {
        match lex(src, "test.cp") {
            Ok(_) => panic!("expected lex error for input {src:?}, got none"),
            Err(e) => e,
        }
    }

    fn integer(value: i128, radix: Radix) -> TokenKind {
        TokenKind::Integer(IntegerLiteral { value, radix })
    }

    fn float(value: f64) -> TokenKind {
        TokenKind::Float(FloatLiteral { value })
    }

    fn kw(k: Keyword) -> TokenKind {
        TokenKind::Keyword(k)
    }

    fn sym(s: Symbol) -> TokenKind {
        TokenKind::Symbol(s)
    }

    #[test]
    fn tokenizes_all_keywords() {
        let src = concat!(
            "let mut fn return if else while for in match ",
            "struct impl import use as pub private protected const enum virtual override final abstract static ",
            "true false self Self ",
            "void char bool ",
            "i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 ",
            "f32 f64 isize usize"
        );
        let expected = vec![
            kw(Let),
            kw(Mut),
            kw(Fn),
            kw(Return),
            kw(If),
            kw(Else),
            kw(While),
            kw(For),
            kw(In),
            kw(Match),
            kw(Struct),
            kw(Impl),
            kw(Import),
            kw(Use),
            kw(As),
            kw(Pub),
            kw(Private),
            kw(Protected),
            kw(Const),
            kw(Enum),
            kw(Virtual),
            kw(Override),
            kw(Final),
            kw(Abstract),
            kw(Static),
            kw(True),
            kw(False),
            kw(SelfKw),
            kw(SelfType),
            kw(Void),
            kw(Char),
            kw(Bool),
            kw(I8),
            kw(I16),
            kw(I32),
            kw(I64),
            kw(I128),
            kw(U8),
            kw(U16),
            kw(U32),
            kw(U64),
            kw(U128),
            kw(F32),
            kw(F64),
            kw(Isize),
            kw(Usize),
            TokenKind::Eof,
        ];
        assert_eq!(kinds(src), expected);
    }

    #[test]
    fn tokenizes_identifiers() {
        assert_eq!(
            kinds("foo bar_1 _1 _bar café 变量"),
            vec![
                TokenKind::Identifier("foo".into()),
                TokenKind::Identifier("bar_1".into()),
                TokenKind::Identifier("_1".into()),
                TokenKind::Identifier("_bar".into()),
                TokenKind::Identifier("café".into()),
                TokenKind::Identifier("变量".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn lone_underscore_is_wildcard_symbol() {
        assert_eq!(
            kinds("_ _x"),
            vec![sym(Symbol::Underscore), TokenKind::Identifier("_x".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn tokenizes_decimal_integers() {
        assert_eq!(
            kinds("0 7 42 9000"),
            vec![
                integer(0, Radix::Decimal),
                integer(7, Radix::Decimal),
                integer(42, Radix::Decimal),
                integer(9000, Radix::Decimal),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn tokenizes_radix_integers() {
        assert_eq!(
            kinds("0x1F 0Xff 0o17 0O17 0b1010 0B101"),
            vec![
                integer(31, Radix::Hex),
                integer(255, Radix::Hex),
                integer(15, Radix::Octal),
                integer(15, Radix::Octal),
                integer(10, Radix::Binary),
                integer(5, Radix::Binary),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    #[allow(clippy::approx_constant)] 
    fn tokenizes_floats() {
        assert_eq!(
            kinds("3.14 .5 1. 1e10 1E10 1.5e-3 2.5E+2"),
            vec![
                float(3.14),
                float(0.5),
                float(1.0),
                float(1e10),
                float(1e10),
                float(0.0015),
                float(250.0),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn distinguishes_int_and_float_after_dot() {
        
        
        assert_eq!(
            kinds("1.5 1.x"),
            vec![float(1.5), float(1.0), TokenKind::Identifier("x".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn digit_then_identifier_lexes_as_two_tokens() {
        assert_eq!(
            kinds("123abc"),
            vec![integer(123, Radix::Decimal), TokenKind::Identifier("abc".into()), TokenKind::Eof]
        );
    }

    #[test]
    fn tokenizes_char_literals() {
        assert_eq!(
            kinds(r"'a' 'z' '0' ' ' '\n' '\t' '\\' '\'' '\x41' '\u{1F600}'"),
            vec![
                TokenKind::Char('a'),
                TokenKind::Char('z'),
                TokenKind::Char('0'),
                TokenKind::Char(' '),
                TokenKind::Char('\n'),
                TokenKind::Char('\t'),
                TokenKind::Char('\\'),
                TokenKind::Char('\''),
                TokenKind::Char('A'),
                TokenKind::Char('😀'),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn tokenizes_string_literals() {
        let src = r#""hello" "" "a\tb\n" "\\\" " "\u{3B1}""#;
        assert_eq!(
            kinds(src),
            vec![
                TokenKind::String("hello".into()),
                TokenKind::String("".into()),
                TokenKind::String("a\tb\n".into()),
                TokenKind::String("\\\" ".into()),
                TokenKind::String("α".into()),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn tokenizes_all_symbols() {
        let src = "( ) { } [ ] ; : , . + - * / % = += -= *= /= %= == != ! < > <= >= && || & | ^ ~ << >> -> => :: ? _";
        let expected = vec![
            sym(Symbol::LeftParen),
            sym(Symbol::RightParen),
            sym(Symbol::LeftBrace),
            sym(Symbol::RightBrace),
            sym(Symbol::LeftBracket),
            sym(Symbol::RightBracket),
            sym(Symbol::Semicolon),
            sym(Symbol::Colon),
            sym(Symbol::Comma),
            sym(Symbol::Dot),
            sym(Symbol::Plus),
            sym(Symbol::Minus),
            sym(Symbol::Star),
            sym(Symbol::Slash),
            sym(Symbol::Percent),
            sym(Symbol::Eq),
            sym(Symbol::PlusEq),
            sym(Symbol::MinusEq),
            sym(Symbol::StarEq),
            sym(Symbol::SlashEq),
            sym(Symbol::PercentEq),
            sym(Symbol::EqEq),
            sym(Symbol::BangEq),
            sym(Symbol::Bang),
            sym(Symbol::Lt),
            sym(Symbol::Gt),
            sym(Symbol::LtEq),
            sym(Symbol::GtEq),
            sym(Symbol::AndAnd),
            sym(Symbol::OrOr),
            sym(Symbol::Amp),
            sym(Symbol::Pipe),
            sym(Symbol::Caret),
            sym(Symbol::Tilde),
            sym(Symbol::Shl),
            sym(Symbol::Shr),
            sym(Symbol::Arrow),
            sym(Symbol::FatArrow),
            sym(Symbol::PathSep),
            sym(Symbol::Question),
            sym(Symbol::Underscore),
            TokenKind::Eof,
        ];
        assert_eq!(kinds(src), expected);
    }

    #[test]
    fn maximal_munch_handles_adjacent_operators() {
        
        assert_eq!(
            kinds("&& && == == -> -> <= >= << >>"),
            vec![
                sym(Symbol::AndAnd),
                sym(Symbol::AndAnd),
                sym(Symbol::EqEq),
                sym(Symbol::EqEq),
                sym(Symbol::Arrow),
                sym(Symbol::Arrow),
                sym(Symbol::LtEq),
                sym(Symbol::GtEq),
                sym(Symbol::Shl),
                sym(Symbol::Shr),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_line_comments() {
        assert_eq!(
            kinds("// hello world\nlet x = 1; // trailing"),
            vec![
                kw(Let),
                TokenKind::Identifier("x".into()),
                sym(Symbol::Eq),
                integer(1, Radix::Decimal),
                sym(Symbol::Semicolon),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn skips_nested_block_comments() {
        assert_eq!(
            kinds("let /* outer /* inner */ still */ x;"),
            vec![
                kw(Let),
                TokenKind::Identifier("x".into()),
                sym(Symbol::Semicolon),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn block_comment_to_eof_is_legal() {
        assert_eq!(kinds("/* nothing but a comment */"), vec![TokenKind::Eof]);
    }

    #[test]
    fn skips_leading_bom() {
        
        assert_eq!(
            kinds("\u{feff}let x = 1;"),
            vec![
                kw(Let),
                TokenKind::Identifier("x".into()),
                sym(Symbol::Eq),
                integer(1, Radix::Decimal),
                sym(Symbol::Semicolon),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn empty_input_yields_only_eof() {
        assert_eq!(kinds(""), vec![TokenKind::Eof]);
    }

    #[test]
    fn tracks_line_and_column() {
        let tokens = lex("let x", "loc.cp").unwrap();
        let let_span = &tokens[0].span;
        let x_loc = &tokens[1].span.start;

        assert_eq!((let_span.start.line, let_span.start.column), (1, 1));
        assert_eq!(let_span.start.offset, 0);
        
        assert_eq!((let_span.end.line, let_span.end.column), (1, 4));
        assert_eq!(let_span.end.offset, 3);
        
        assert_eq!((x_loc.line, x_loc.column), (1, 5));
        assert_eq!(x_loc.offset, 4);
    }

    #[test]
    fn tracks_multiline_locations() {
        let src = "a\nbb\n  ccc";
        let tokens = lex(src, "lines.cp").unwrap();
        let locs: Vec<(u32, u32)> = tokens
            .iter()
            .map(|t| (t.span.start.line, t.span.start.column))
            .collect();
        
        assert_eq!(locs, vec![(1, 1), (2, 1), (3, 3), (3, 6)]);
    }

    #[test]
    fn tracks_crlf_line_endings() {
        let tokens = lex("a\r\nb\r\n", "crlf.cp").unwrap();
        assert_eq!(tokens[0].span.start.line, 1);
        assert_eq!(tokens[1].span.start.line, 2);
        assert_eq!(tokens[1].span.start.column, 1);
        assert_eq!(tokens[2].span.start.line, 3);
    }

    #[test]
    fn eof_has_zero_width_span_at_end() {
        let tokens = lex("hi", "eof.cp").unwrap();
        let eof = tokens.last().unwrap();
        assert!(eof.is_eof());
        assert_eq!(eof.span.start, eof.span.end);
        assert_eq!(eof.span.start.offset, 2);
    }

    #[test]
    fn unterminated_string_reports_error() {
        assert_eq!(
            error_of("\"abc"),
            LexError::UnterminatedString {
                location: SourceLocation::new("test.cp", 1, 1, 0)
            }
        );
    }

    #[test]
    fn newline_terminates_string_literal() {
        assert!(matches!(error_of("\"abc\nlet"), LexError::UnterminatedString { .. }));
    }

    #[test]
    fn unterminated_char_reports_error() {
        assert!(matches!(error_of("'a"), LexError::UnterminatedChar { .. }));
    }

    #[test]
    fn empty_char_literal_is_error() {
        assert!(matches!(
            error_of("''"),
            LexError::EmptyCharLiteral { .. }
        ));
    }

    #[test]
    fn multi_char_literal_is_error() {
        assert!(matches!(
            error_of("'ab'"),
            LexError::MultiCharLiteral { .. }
        ));
    }

    #[test]
    fn invalid_escape_is_error() {
        assert!(matches!(
            error_of(r"'\%'"),
            LexError::InvalidEscape { escape, .. } if escape == "%"
        ));
        assert!(matches!(
            error_of(r#""\%""#),
            LexError::InvalidEscape { escape, .. } if escape == "%"
        ));
    }

    #[test]
    fn unterminated_block_comment_is_error() {
        assert_eq!(
            error_of("/* oops"),
            LexError::UnterminatedBlockComment {
                location: SourceLocation::new("test.cp", 1, 1, 0)
            }
        );
    }

    #[test]
    fn invalid_radix_literal_is_error() {
        assert!(matches!(error_of("0x"), LexError::InvalidRadixLiteral { .. }));
        assert!(matches!(error_of("0b2"), LexError::InvalidRadixLiteral { .. }));
        assert!(matches!(error_of("0o9"), LexError::InvalidRadixLiteral { .. }));
        assert!(matches!(error_of("0xG"), LexError::InvalidRadixLiteral { .. }));
    }

    #[test]
    fn integer_overflow_is_error() {
        assert!(matches!(
            error_of("99999999999999999999999999999999999999999999999999"),
            LexError::IntegerOverflow { .. }
        ));
        
        assert!(matches!(
            error_of("0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF"),
            LexError::IntegerOverflow { .. }
        ));
    }

    #[test]
    fn invalid_character_is_error() {
        assert_eq!(
            error_of("#"),
            LexError::InvalidCharacter {
                location: SourceLocation::new("test.cp", 1, 1, 0),
                c: '#'
            }
        );
    }

    #[test]
    fn hex_escape_requires_digits() {
        assert!(matches!(error_of(r"'\x'"), LexError::InvalidHexEscape { .. }));
    }

    #[test]
    fn unicode_escape_requires_braces_and_range() {
        assert!(matches!(
            error_of(r"'\u12'"),
            LexError::InvalidUnicodeEscape { .. }
        ));
        
        assert!(matches!(
            error_of(r"'\u{D800}'"),
            LexError::InvalidUnicodeEscape { .. }
        ));
    }

    #[test]
    #[allow(clippy::approx_constant)] 
    fn tokenizes_realistic_program() {
        let src = r#"
import std.io;

fn main() -> i32 {
    let x: i32 = 10;
    let mut y: f64 = 3.14;
    if x > 5 {
        return x;
    } else {
        return 0;
    }
}
"#;
        assert_eq!(
            kinds(src),
            vec![
                kw(Import),
                TokenKind::Identifier("std".into()),
                sym(Symbol::Dot),
                TokenKind::Identifier("io".into()),
                sym(Symbol::Semicolon),
                kw(Fn),
                TokenKind::Identifier("main".into()),
                sym(Symbol::LeftParen),
                sym(Symbol::RightParen),
                sym(Symbol::Arrow),
                kw(I32),
                sym(Symbol::LeftBrace),
                kw(Let),
                TokenKind::Identifier("x".into()),
                sym(Symbol::Colon),
                kw(I32),
                sym(Symbol::Eq),
                integer(10, Radix::Decimal),
                sym(Symbol::Semicolon),
                kw(Let),
                kw(Mut),
                TokenKind::Identifier("y".into()),
                sym(Symbol::Colon),
                kw(F64),
                sym(Symbol::Eq),
                float(3.14),
                sym(Symbol::Semicolon),
                kw(If),
                TokenKind::Identifier("x".into()),
                sym(Symbol::Gt),
                integer(5, Radix::Decimal),
                sym(Symbol::LeftBrace),
                kw(Return),
                TokenKind::Identifier("x".into()),
                sym(Symbol::Semicolon),
                sym(Symbol::RightBrace),
                kw(Else),
                sym(Symbol::LeftBrace),
                kw(Return),
                integer(0, Radix::Decimal),
                sym(Symbol::Semicolon),
                sym(Symbol::RightBrace),
                sym(Symbol::RightBrace),
                TokenKind::Eof,
            ]
        );
    }

    #[test]
    fn iterator_yields_none_after_eof() {
        let mut lexer = Lexer::new("x", "iter.cp");
        let first = lexer.next().unwrap().unwrap();
        assert!(!first.is_eof());
        let second = lexer.next().unwrap().unwrap();
        assert!(second.is_eof());
        assert!(lexer.next().is_none());
    }
}
