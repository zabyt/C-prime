








use crate::ast::*;
use crate::token::{Keyword, SourceLocation, SourceSpan, Symbol, Token, TokenKind};



#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum ParseError {
    #[error("{span}: expected {expected}, found {found}")]
    UnexpectedToken {
        span: SourceSpan,
        expected: String,
        found: String,
    },
    #[error("{span}: unexpected end of file, expected {expected}")]
    UnexpectedEof { span: SourceSpan, expected: String },
}





pub fn parse_tokens(tokens: Vec<Token>) -> Result<Program, ParseError> {
    Parser::new(tokens).parse_program()
}


pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    
    pub fn new(tokens: Vec<Token>) -> Parser {
        Parser { tokens, pos: 0 }
    }

    
    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut items = Vec::new();
        while !self.at_eof() {
            items.push(self.parse_item()?);
        }
        let span = if items.is_empty() {
            let loc = self.tokens[0].span.start.clone();
            SourceSpan::new(loc.clone(), loc)
        } else {
            let start = items[0].span().start.clone();
            let end = items[items.len() - 1].span().end.clone();
            SourceSpan::new(start, end)
        };
        Ok(Program { items, span })
    }

    
    
    

    
    fn peek_token(&self) -> &Token {
        &self.tokens[self.pos]
    }

    
    fn peek_kind(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    
    fn at_eof(&self) -> bool {
        self.pos >= self.tokens.len() || self.tokens[self.pos].is_eof()
    }

    
    fn at_symbol(&self, s: Symbol) -> bool {
        matches!(self.peek_kind(), TokenKind::Symbol(sym) if *sym == s)
    }

    
    fn at_keyword(&self, k: Keyword) -> bool {
        matches!(self.peek_kind(), TokenKind::Keyword(kw) if *kw == k)
    }

    
    fn advance(&mut self) {
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
    }

    
    fn eat_symbol(&mut self, s: Symbol) -> bool {
        if self.at_symbol(s) {
            self.advance();
            true
        } else {
            false
        }
    }

    
    fn eat_keyword(&mut self, k: Keyword) -> bool {
        if self.at_keyword(k) {
            self.advance();
            true
        } else {
            false
        }
    }

    
    fn expect_symbol(&mut self, s: Symbol, what: &str) -> Result<Token, ParseError> {
        if self.at_symbol(s) {
            let tok = self.tokens[self.pos].clone();
            self.advance();
            Ok(tok)
        } else {
            Err(self.error_expected(what))
        }
    }

    
    fn expect_keyword(&mut self, k: Keyword, what: &str) -> Result<Token, ParseError> {
        if self.at_keyword(k) {
            let tok = self.tokens[self.pos].clone();
            self.advance();
            Ok(tok)
        } else {
            Err(self.error_expected(what))
        }
    }

    
    fn expect_identifier(&mut self, what: &str) -> Result<String, ParseError> {
        match self.peek_kind() {
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(name)
            }
            _ => Err(self.error_expected(what)),
        }
    }

    
    fn prev_end(&self) -> SourceLocation {
        if self.pos == 0 {
            self.tokens[0].span.start.clone()
        } else {
            self.tokens[self.pos - 1].span.end.clone()
        }
    }

    
    fn error_expected(&self, expected: &str) -> ParseError {
        let tok = self.peek_token();
        if tok.is_eof() {
            ParseError::UnexpectedEof {
                span: tok.span.clone(),
                expected: expected.to_string(),
            }
        } else {
            ParseError::UnexpectedToken {
                span: tok.span.clone(),
                expected: expected.to_string(),
                found: tok.kind.describe(),
            }
        }
    }

    
    fn span_from(&self, start: SourceLocation) -> SourceSpan {
        SourceSpan::new(start, self.prev_end())
    }

    
    
    

    fn parse_item(&mut self) -> Result<Item, ParseError> {
        match self.peek_kind() {
            TokenKind::Keyword(Keyword::Import) => self.parse_import().map(Item::Import),
            TokenKind::Keyword(Keyword::Fn) => self.parse_function().map(Item::Function),
            TokenKind::Keyword(Keyword::Abstract)
            | TokenKind::Keyword(Keyword::Final)
            | TokenKind::Keyword(Keyword::Struct)
            | TokenKind::Keyword(Keyword::Class) => self.parse_struct().map(Item::Struct),
            TokenKind::Keyword(Keyword::Enum) => self.parse_enum().map(Item::Enum),
            TokenKind::Keyword(Keyword::Using) => self.parse_type_alias().map(Item::TypeAlias),
            TokenKind::Keyword(Keyword::Const) => self.parse_const_decl().map(Item::Const),
            TokenKind::Keyword(Keyword::Impl) => self.parse_impl().map(Item::Impl),
            TokenKind::Keyword(Keyword::Namespace) => self.parse_namespace().map(Item::Namespace),
            TokenKind::Keyword(Keyword::Extern) => self.parse_extern_function().map(Item::ExternFunction),
            _ => Err(self.error_expected(
                "a top-level item (`import`, `extern`, `fn`, `struct`, `class`, `enum`, `using`, `const`, `impl`, or `namespace`)",
            )),
        }
    }

    fn parse_import(&mut self) -> Result<ImportDecl, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::Import, "`import`")?;
        let mut path = Vec::new();
        path.push(self.expect_identifier("a module name")?);
        while self.eat_symbol(Symbol::Dot) {
            path.push(self.expect_identifier("a module name")?);
        }
        self.expect_symbol(Symbol::Semicolon, "`;`")?;
        Ok(ImportDecl {
            path,
            span: self.span_from(start),
        })
    }

    fn parse_visibility(&mut self) -> crate::ast::Visibility {
        if self.eat_keyword(Keyword::Pub) {
            crate::ast::Visibility::Public
        } else if self.eat_keyword(Keyword::Private) {
            crate::ast::Visibility::Private
        } else if self.eat_keyword(Keyword::Protected) {
            crate::ast::Visibility::Protected
        } else {
            crate::ast::Visibility::Public
        }
    }

    fn starts_method_like(&self) -> bool {
        let mut pos = self.pos;
        loop {
            match self.tokens.get(pos).map(|t| &t.kind) {
                Some(TokenKind::Keyword(Keyword::Pub | Keyword::Private | Keyword::Protected)) => {
                    pos += 1;
                }
                Some(TokenKind::Keyword(Keyword::Virtual | Keyword::Override | Keyword::Final | Keyword::Abstract | Keyword::Static)) => {
                    pos += 1;
                }
                Some(TokenKind::Keyword(Keyword::Fn)) => return true,
                _ => return false,
            }
        }
    }

    fn parse_function(&mut self) -> Result<Function, ParseError> {
        let start = self.peek_token().span.start.clone();
        let visibility = self.parse_visibility();
        let is_virtual = self.eat_keyword(Keyword::Virtual);
        let is_override = self.eat_keyword(Keyword::Override);
        let is_final = self.eat_keyword(Keyword::Final);
        let is_abstract = self.eat_keyword(Keyword::Abstract);
        let is_static = self.eat_keyword(Keyword::Static);
        self.expect_keyword(Keyword::Fn, "`fn`")?;
        let name = self.expect_identifier("a function name")?;
        let generics = if self.at_symbol(Symbol::LeftBracket) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };
        self.expect_symbol(Symbol::LeftParen, "`(`")?;
        let mut params = Vec::new();
        if !self.at_symbol(Symbol::RightParen) {
            loop {
                params.push(self.parse_param()?);
                if !self.eat_symbol(Symbol::Comma) {
                    break;
                }
            }
        }
        self.expect_symbol(Symbol::RightParen, "`)`")?;
        let is_const = self.eat_keyword(Keyword::Const);
        let return_ty = if self.eat_symbol(Symbol::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let body = if self.eat_symbol(Symbol::Semicolon) {
            Block { stmts: Vec::new(), span: self.span_from(start.clone()) }
        } else {
            self.parse_block()?
        };
        Ok(Function {
            name,
            generics,
            params,
            visibility,
            is_virtual,
            is_override,
            is_final,
            is_abstract,
            is_static,
            is_const,
            return_ty,
            body,
            span: self.span_from(start),
        })
    }

    
    
    fn parse_extern_function(&mut self) -> Result<ExternFunctionDecl, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::Extern, "`extern`")?;
        self.expect_keyword(Keyword::Fn, "`fn`")?;
        let name = self.expect_identifier("an extern function name")?;
        self.expect_symbol(Symbol::LeftParen, "`(`")?;
        let mut params = Vec::new();
        let mut is_variadic = false;
        if !self.at_symbol(Symbol::RightParen) {
            loop {
                if self.eat_symbol(Symbol::Ellipsis) {
                    is_variadic = true;
                    break;
                }
                params.push(self.parse_param()?);
                if !self.eat_symbol(Symbol::Comma) {
                    break;
                }
            }
        }
        self.expect_symbol(Symbol::RightParen, "`)`")?;
        let return_ty = if self.eat_symbol(Symbol::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect_symbol(Symbol::Semicolon, "`;`")?;
        Ok(ExternFunctionDecl {
            name,
            params,
            return_ty,
            is_variadic,
            span: self.span_from(start),
        })
    }

    fn parse_param(&mut self) -> Result<Param, ParseError> {
        let start = self.peek_token().span.start.clone();
        if self.eat_keyword(Keyword::SelfKw) {
            let ty = Type::Named {
                name: "Self".to_string(),
                args: Vec::new(),
                span: self.span_from(start.clone()),
            };
            return Ok(Param {
                name: "self".to_string(),
                ty,
                span: self.span_from(start),
            });
        }
        if self.at_symbol(Symbol::Amp) {
            let borrow_start = self.peek_token().span.start.clone();
            self.advance();
            let mutability = if self.eat_keyword(Keyword::Mut) {
                PointerMutability::Mut
            } else {
                PointerMutability::Const
            };
            if self.eat_keyword(Keyword::SelfKw) {
                let pointee = Type::Named {
                    name: "Self".to_string(),
                    args: Vec::new(),
                    span: self.span_from(borrow_start.clone()),
                };
                return Ok(Param {
                    name: "self".to_string(),
                    ty: Type::Pointer {
                        pointee: Box::new(pointee),
                        mutability,
                        span: self.span_from(borrow_start),
                    },
                    span: self.span_from(start),
                });
            }
            return Err(self.error_expected("`self`, `&self`, or `&mut self`"));
        }
        let name = self.expect_identifier("a parameter name")?;
        self.expect_symbol(Symbol::Colon, "`:`")?;
        let ty = self.parse_type()?;
        Ok(Param {
            name,
            ty,
            span: self.span_from(start),
        })
    }

    
    fn parse_generic_params(&mut self) -> Result<Vec<String>, ParseError> {
        self.expect_symbol(Symbol::LeftBracket, "`[`")?;
        let mut params = Vec::new();
        if !self.at_symbol(Symbol::RightBracket) {
            loop {
                params.push(self.expect_identifier("a type parameter name")?);
                if !self.eat_symbol(Symbol::Comma) {
                    break;
                }
            }
        }
        self.expect_symbol(Symbol::RightBracket, "`]`")?;
        Ok(params)
    }

    fn parse_struct(&mut self) -> Result<StructDecl, ParseError> {
        let start = self.peek_token().span.start.clone();
        let is_abstract = self.eat_keyword(Keyword::Abstract);
        let is_final = self.eat_keyword(Keyword::Final);
        if self.at_keyword(Keyword::Class) {
            self.advance();
        } else {
            self.expect_keyword(Keyword::Struct, "`struct` or `class`")?;
        }
        let name = self.expect_identifier("a struct name")?;
        let generics = if self.at_symbol(Symbol::LeftBracket) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };
        let base = if self.eat_symbol(Symbol::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect_symbol(Symbol::LeftBrace, "`{`")?;
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        while !self.at_symbol(Symbol::RightBrace) {
            if self.at_eof() {
                return Err(self.error_expected("a struct field, method, or `}`"));
            }
            if self.starts_method_like() {
                methods.push(self.parse_function()?);
            } else {
                fields.push(self.parse_field()?);
                if self.at_symbol(Symbol::RightBrace) {
                    break;
                }
                if self.at_symbol(Symbol::Comma) {
                    self.advance();
                    if self.at_symbol(Symbol::RightBrace) {
                        break;
                    }
                }
            }
        }
        self.expect_symbol(Symbol::RightBrace, "`}`")?;
        Ok(StructDecl {
            name,
            generics,
            base,
            is_final,
            is_abstract,
            fields,
            methods,
            span: self.span_from(start),
        })
    }

    fn parse_enum(&mut self) -> Result<EnumDecl, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::Enum, "`enum`")?;
        if self.at_keyword(Keyword::Class) {
            self.advance();
        }
        let name = self.expect_identifier("an enum name")?;
        let base = if self.eat_symbol(Symbol::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        self.expect_symbol(Symbol::LeftBrace, "`{`")?;
        let mut variants = Vec::new();
        while !self.at_symbol(Symbol::RightBrace) {
            if self.at_eof() {
                return Err(self.error_expected("an enum variant or `}`"));
            }
            let variant_start = self.peek_token().span.start.clone();
            let variant_name = self.expect_identifier("an enum variant name")?;
            let value = if self.eat_symbol(Symbol::Eq) {
                Some(self.parse_expr()?)
            } else {
                None
            };
            variants.push(EnumVariant {
                name: variant_name,
                value,
                span: self.span_from(variant_start),
            });
            if !self.eat_symbol(Symbol::Comma) {
                break;
            }
        }
        self.expect_symbol(Symbol::RightBrace, "`}`")?;
        Ok(EnumDecl {
            name,
            base,
            variants,
            span: self.span_from(start),
        })
    }

    fn parse_type_alias(&mut self) -> Result<TypeAliasDecl, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::Using, "`using`")?;
        let name = self.expect_identifier("a type alias name")?;
        self.expect_symbol(Symbol::Eq, "`=`")?;
        let target = self.parse_type()?;
        self.expect_symbol(Symbol::Semicolon, "`;`")?;
        Ok(TypeAliasDecl {
            name,
            target,
            span: self.span_from(start),
        })
    }

    fn parse_const_decl(&mut self) -> Result<ConstDecl, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::Const, "`const`")?;
        let name = self.expect_identifier("a constant name")?;
        self.expect_symbol(Symbol::Colon, "`:`")?;
        let ty = self.parse_type()?;
        self.expect_symbol(Symbol::Eq, "`=`")?;
        let value = self.parse_expr()?;
        self.expect_symbol(Symbol::Semicolon, "`;`")?;
        Ok(ConstDecl {
            name,
            ty,
            value,
            span: self.span_from(start),
        })
    }

    fn parse_field(&mut self) -> Result<FieldDecl, ParseError> {
        let start = self.peek_token().span.start.clone();
        let visibility = self.parse_visibility();
        let name = self.expect_identifier("a field name")?;
        self.expect_symbol(Symbol::Colon, "`:`")?;
        let ty = self.parse_type()?;
        Ok(FieldDecl {
            name,
            visibility,
            ty,
            span: self.span_from(start),
        })
    }

    fn parse_impl(&mut self) -> Result<ImplBlock, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::Impl, "`impl`")?;
        let type_name = self.expect_identifier("a type name")?;
        let generics = if self.at_symbol(Symbol::LeftBracket) {
            self.parse_generic_params()?
        } else {
            Vec::new()
        };
        self.expect_symbol(Symbol::LeftBrace, "`{`")?;
        let mut methods = Vec::new();
        while !self.at_symbol(Symbol::RightBrace) {
            if self.at_eof() {
                return Err(self.error_expected("a method or `}`"));
            }
            methods.push(self.parse_function()?);
        }
        self.expect_symbol(Symbol::RightBrace, "`}`")?;
        Ok(ImplBlock {
            type_name,
            generics,
            methods,
            span: self.span_from(start),
        })
    }

    fn parse_namespace(&mut self) -> Result<NamespaceDecl, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::Namespace, "`namespace`")?;
        let name = self.expect_identifier("a namespace name")?;
        self.expect_symbol(Symbol::LeftBrace, "`{`")?;
        let mut items = Vec::new();
        while !self.at_symbol(Symbol::RightBrace) {
            if self.at_eof() {
                return Err(self.error_expected("an item or `}`"));
            }
            items.push(self.parse_item()?);
        }
        self.expect_symbol(Symbol::RightBrace, "`}`")?;
        Ok(NamespaceDecl {
            name,
            items,
            span: self.span_from(start),
        })
    }

    
    
    

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        
        if self.at_symbol(Symbol::LeftBracket) {
            let start = self.peek_token().span.start.clone();
            self.advance(); 
            let element = self.parse_type()?;
            if self.at_symbol(Symbol::Semicolon) {
                self.advance(); 
                let length = self.parse_expr()?;
                self.expect_symbol(Symbol::RightBracket, "`]`")?;
                return Ok(Type::Array {
                    element: Box::new(element),
                    length: Box::new(length),
                    span: self.span_from(start),
                });
            }
            
            
            
            
            
            return Err(self.error_expected("`;` in array type (expected `[T; N]`)"));
        }

        if self.at_symbol(Symbol::Amp) {
            let start = self.peek_token().span.start.clone();
            self.advance();
            let mutability = if self.eat_keyword(Keyword::Mut) {
                PointerMutability::Mut
            } else if self.eat_keyword(Keyword::Const) {
                PointerMutability::Const
            } else {
                PointerMutability::Const
            };
            let pointee = self.parse_type()?;
            return Ok(Type::Pointer {
                pointee: Box::new(pointee),
                mutability,
                span: self.span_from(start),
            });
        }

        if self.at_symbol(Symbol::Star) {
            let start = self.peek_token().span.start.clone();
            self.advance();
            let mutability = if self.eat_keyword(Keyword::Mut) {
                PointerMutability::Mut
            } else if self.eat_keyword(Keyword::Const) {
                PointerMutability::Const
            } else {
                return Err(self.error_expected("`mut` or `const` after `*`"));
            };
            let pointee = self.parse_type()?;
            return Ok(Type::Pointer {
                pointee: Box::new(pointee),
                mutability,
                span: self.span_from(start),
            });
        }

        let start = self.peek_token().span.start.clone();
        if let TokenKind::Keyword(kw) = self.peek_kind() {
            if let Some(prim) = PrimitiveType::from_keyword(*kw) {
                self.advance();
                return Ok(Type::Primitive(prim));
            }
            if *kw == Keyword::SelfType {
                self.advance();
                return Ok(Type::Named {
                    name: "Self".to_string(),
                    args: Vec::new(),
                    span: self.span_from(start),
                });
            }
            return Err(self.error_expected("a type"));
        }
        if let TokenKind::Identifier(_) = self.peek_kind() {
            let name = self.expect_identifier("a type name")?;
            let args = if self.at_symbol(Symbol::LeftBracket) {
                self.parse_type_args()?
            } else {
                Vec::new()
            };
            return Ok(Type::Named {
                name,
                args,
                span: self.span_from(start),
            });
        }
        Err(self.error_expected("a type"))
    }

    
    fn parse_type_args(&mut self) -> Result<Vec<Type>, ParseError> {
        self.expect_symbol(Symbol::LeftBracket, "`[`")?;
        let mut args = Vec::new();
        if !self.at_symbol(Symbol::RightBracket) {
            loop {
                args.push(self.parse_type()?);
                if !self.eat_symbol(Symbol::Comma) {
                    break;
                }
            }
        }
        self.expect_symbol(Symbol::RightBracket, "`]`")?;
        Ok(args)
    }

    
    
    

    fn parse_block(&mut self) -> Result<Block, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_symbol(Symbol::LeftBrace, "`{`")?;
        let mut stmts = Vec::new();
        while !self.at_symbol(Symbol::RightBrace) {
            if self.at_eof() {
                return Err(self.error_expected("a statement or `}`"));
            }
            stmts.push(self.parse_stmt()?);
        }
        self.expect_symbol(Symbol::RightBrace, "`}`")?;
        Ok(Block {
            stmts,
            span: self.span_from(start),
        })
    }

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        match self.peek_kind() {
            TokenKind::Keyword(Keyword::Let) => self.parse_let().map(Stmt::Let),
            TokenKind::Keyword(Keyword::If) => self.parse_if().map(Stmt::If),
            TokenKind::Keyword(Keyword::While) => self.parse_while().map(Stmt::While),
            TokenKind::Keyword(Keyword::For) => self.parse_for(),
            TokenKind::Keyword(Keyword::Switch) => self.parse_switch().map(Stmt::Switch),
            TokenKind::Keyword(Keyword::Break) => self.parse_break().map(Stmt::Break),
            TokenKind::Keyword(Keyword::Continue) => self.parse_continue().map(Stmt::Continue),
            TokenKind::Symbol(Symbol::LeftBrace) => self.parse_block().map(Stmt::Block),
            _ => {
                let expr = self.parse_expr()?;
                if matches!(&expr, Expr::Match { .. }) {
                    
                    
                    self.eat_symbol(Symbol::Semicolon);
                } else {
                    self.expect_symbol(Symbol::Semicolon, "`;`")?;
                }
                Ok(Stmt::Expr(expr))
            }
        }
    }

    fn parse_switch(&mut self) -> Result<SwitchStmt, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::Switch, "`switch`")?;
        let value = self.parse_expr_no_struct()?;
        self.expect_symbol(Symbol::LeftBrace, "`{`")?;
        let mut arms = Vec::new();
        while !self.at_symbol(Symbol::RightBrace) {
            if self.at_eof() {
                return Err(self.error_expected("a `case` arm or `}`"));
            }
            if self.at_keyword(Keyword::Case) {
                arms.push(self.parse_switch_case()?);
            } else if self.at_keyword(Keyword::Default) {
                arms.push(self.parse_switch_default()?);
            } else {
                return Err(self.error_expected("a `case` or `default` arm"));
            }
        }
        self.expect_symbol(Symbol::RightBrace, "`}`")?;
        Ok(SwitchStmt {
            value,
            arms,
            span: self.span_from(start),
        })
    }

    fn parse_switch_case(&mut self) -> Result<SwitchArm, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::Case, "`case`")?;
        let pattern = self.parse_expr()?;
        self.expect_symbol(Symbol::Colon, "`:`")?;
        let mut body = Vec::new();
        while !self.at_symbol(Symbol::RightBrace)
            && !self.at_keyword(Keyword::Case)
            && !self.at_keyword(Keyword::Default)
        {
            body.push(self.parse_stmt()?);
        }
        Ok(SwitchArm {
            pattern: Some(pattern),
            body,
            span: self.span_from(start),
        })
    }

    fn parse_switch_default(&mut self) -> Result<SwitchArm, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::Default, "`default`")?;
        self.expect_symbol(Symbol::Colon, "`:`")?;
        let mut body = Vec::new();
        while !self.at_symbol(Symbol::RightBrace)
            && !self.at_keyword(Keyword::Case)
            && !self.at_keyword(Keyword::Default)
        {
            body.push(self.parse_stmt()?);
        }
        Ok(SwitchArm {
            pattern: None,
            body,
            span: self.span_from(start),
        })
    }

    fn parse_break(&mut self) -> Result<BreakStmt, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::Break, "`break`")?;
        self.expect_symbol(Symbol::Semicolon, "`;`")?;
        Ok(BreakStmt { span: self.span_from(start) })
    }

    fn parse_continue(&mut self) -> Result<ContinueStmt, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::Continue, "`continue`")?;
        self.expect_symbol(Symbol::Semicolon, "`;`")?;
        Ok(ContinueStmt { span: self.span_from(start) })
    }

    fn parse_let(&mut self) -> Result<LetStmt, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::Let, "`let`")?;
        let mutable = self.eat_keyword(Keyword::Mut);
        let name = self.expect_identifier("a variable name")?;
        let ty = if self.eat_symbol(Symbol::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        let init = if self.eat_symbol(Symbol::Eq) {
            Some(self.parse_expr()?)
        } else {
            None
        };
        self.expect_symbol(Symbol::Semicolon, "`;`")?;
        Ok(LetStmt {
            name,
            mutable,
            ty,
            init,
            span: self.span_from(start),
        })
    }

    fn parse_if(&mut self) -> Result<IfStmt, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::If, "`if`")?;
        let condition = self.parse_expr_no_struct()?;
        let then_block = self.parse_block()?;
        let else_branch = if self.eat_keyword(Keyword::Else) {
            if self.at_keyword(Keyword::If) {
                Some(ElseBranch::If(Box::new(self.parse_if()?)))
            } else {
                Some(ElseBranch::Block(self.parse_block()?))
            }
        } else {
            None
        };
        Ok(IfStmt {
            condition,
            then_block,
            else_branch,
            span: self.span_from(start),
        })
    }

    fn parse_while(&mut self) -> Result<WhileStmt, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::While, "`while`")?;
        let condition = self.parse_expr_no_struct()?;
        let body = self.parse_block()?;
        Ok(WhileStmt {
            condition,
            body,
            span: self.span_from(start),
        })
    }

    fn parse_for(&mut self) -> Result<Stmt, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::For, "`for`")?;
        let variable = self.expect_identifier("a loop variable")?;
        self.expect_keyword(Keyword::In, "`in`")?;
        let first = self.parse_expr_no_struct()?;

        
        let is_range = self.at_symbol(Symbol::DotDot);
        let is_inclusive_range = self.at_symbol(Symbol::DotDotEq);

        if is_range || is_inclusive_range {
            let inclusive = is_inclusive_range;
            self.advance(); 
            let end = self.parse_expr_no_struct()?;
            let body = self.parse_block()?;
            let range_span = self.span_from(start.clone());

            let start_var = "__range_start".to_string();
            let end_var = "__range_end".to_string();

            let cond = if inclusive {
                Expr::Binary {
                    op: BinaryOp::LtEq,
                    lhs: Box::new(Expr::Var { name: variable.clone(), span: range_span.clone() }),
                    rhs: Box::new(Expr::Var { name: end_var.clone(), span: range_span.clone() }),
                    span: range_span.clone(),
                }
            } else {
                Expr::Binary {
                    op: BinaryOp::Lt,
                    lhs: Box::new(Expr::Var { name: variable.clone(), span: range_span.clone() }),
                    rhs: Box::new(Expr::Var { name: end_var.clone(), span: range_span.clone() }),
                    span: range_span.clone(),
                }
            };

            let mut stmts = Vec::new();
            stmts.push(Stmt::Let(LetStmt {
                name: start_var.clone(),
                mutable: false,
                ty: None,
                init: Some(first),
                span: range_span.clone(),
            }));
            stmts.push(Stmt::Let(LetStmt {
                name: end_var.clone(),
                mutable: false,
                ty: None,
                init: Some(end),
                span: range_span.clone(),
            }));
            stmts.push(Stmt::Let(LetStmt {
                name: variable.clone(),
                mutable: true,
                ty: None,
                init: Some(Expr::Var { name: start_var, span: range_span.clone() }),
                span: range_span.clone(),
            }));
            let mut while_body_stmts = body.stmts;
            while_body_stmts.push(Stmt::Expr(Expr::Assign {
                target: Box::new(Expr::Var { name: variable.clone(), span: range_span.clone() }),
                op: AssignOp::Assign,
                value: Box::new(Expr::Binary {
                    op: BinaryOp::Add,
                    lhs: Box::new(Expr::Var { name: variable, span: range_span.clone() }),
                    rhs: Box::new(Expr::Literal(LiteralExpr::Integer(1), range_span.clone())),
                    span: range_span.clone(),
                }),
                span: range_span.clone(),
            }));
            stmts.push(Stmt::While(WhileStmt {
                condition: cond,
                body: Block {
                    stmts: while_body_stmts,
                    span: body.span,
                },
                span: range_span.clone(),
            }));

            let end_loc = self.prev_end();
            Ok(Stmt::Block(Block {
                stmts,
                span: SourceSpan::new(start, end_loc),
            }))
        } else {
            let iterable = first;
            let body = self.parse_block()?;
            Ok(Stmt::For(ForStmt {
                variable,
                iterable,
                body,
                span: self.span_from(start),
            }))
        }
    }

    
    
    

    
    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr_flags(false)
    }

    
    
    fn parse_expr_no_struct(&mut self) -> Result<Expr, ParseError> {
        self.parse_expr_flags(true)
    }

    fn parse_expr_flags(&mut self, no_struct: bool) -> Result<Expr, ParseError> {
        let start = self.peek_token().span.start.clone();
        let lhs = self.parse_binary(1, no_struct)?;
        if let Some(op) = self.cur_assign_op() {
            self.advance();
            let value = self.parse_expr_flags(no_struct)?;
            let end = self.prev_end();
            return Ok(Expr::Assign {
                target: Box::new(lhs),
                op,
                value: Box::new(value),
                span: SourceSpan::new(start, end),
            });
        }
        Ok(lhs)
    }

    
    
    fn parse_binary(&mut self, min_prec: u8, no_struct: bool) -> Result<Expr, ParseError> {
        let start = self.peek_token().span.start.clone();
        let mut lhs = self.parse_unary(no_struct)?;
        while let Some((op, prec)) = self.cur_binary_op() {
            if prec < min_prec {
                break;
            }
            self.advance();
            let rhs = self.parse_binary(prec + 1, no_struct)?;
            let end = self.prev_end();
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
                span: SourceSpan::new(start.clone(), end),
            };
        }
        Ok(lhs)
    }

    fn parse_unary(&mut self, no_struct: bool) -> Result<Expr, ParseError> {
        let start = self.peek_token().span.start.clone();
        let op = if self.eat_symbol(Symbol::Minus) {
            Some(UnaryOp::Neg)
        } else if self.eat_symbol(Symbol::Bang) {
            Some(UnaryOp::Not)
        } else if self.eat_symbol(Symbol::Tilde) {
            Some(UnaryOp::BitNot)
        } else if self.eat_symbol(Symbol::Star) {
            Some(UnaryOp::Deref)
        } else if self.eat_symbol(Symbol::Amp) {
            if self.eat_keyword(Keyword::Mut) {
                Some(UnaryOp::AddrOfMut)
            } else {
                Some(UnaryOp::AddrOf)
            }
        } else {
            None
        };
        match op {
            Some(op) => {
                let operand = self.parse_unary(no_struct)?;
                let end = self.prev_end();
                Ok(Expr::Unary {
                    op,
                    operand: Box::new(operand),
                    span: SourceSpan::new(start, end),
                })
            }
            None => self.parse_postfix(no_struct),
        }
    }

    fn parse_postfix(&mut self, no_struct: bool) -> Result<Expr, ParseError> {
        let mut expr = self.parse_primary(no_struct)?;
        loop {
            if self.at_symbol(Symbol::LeftParen) {
                self.advance();
                let args = self.parse_call_args()?;
                let end = self.prev_end();
                let start = expr.span().start.clone();
                expr = Expr::Call {
                    callee: Box::new(expr),
                    args,
                    span: SourceSpan::new(start, end),
                };
            } else if self.eat_symbol(Symbol::Dot) {
                let member = self.expect_identifier("a field or method name")?;
                let start = expr.span().start.clone();
                if self.at_symbol(Symbol::LeftParen) {
                    self.advance();
                    let args = self.parse_call_args()?;
                    let end = self.prev_end();
                    expr = Expr::MethodCall {
                        receiver: Box::new(expr),
                        method: member,
                        args,
                        span: SourceSpan::new(start, end),
                    };
                } else {
                    let end = self.prev_end();
                    expr = Expr::FieldAccess {
                        base: Box::new(expr),
                        field: member,
                        span: SourceSpan::new(start, end),
                    };
                }
            } else if self.eat_symbol(Symbol::LeftBracket) {
                let index = self.parse_expr()?;
                self.expect_symbol(Symbol::RightBracket, "`]`")?;
                let end = self.prev_end();
                let start = expr.span().start.clone();
                expr = Expr::Index {
                    base: Box::new(expr),
                    index: Box::new(index),
                    span: SourceSpan::new(start, end),
                };
            } else if self.eat_symbol(Symbol::PathSep) {
                let segment = self.expect_identifier("a path segment")?;
                let end = self.prev_end();
                let start = expr.span().start.clone();
                expr = match expr {
                    Expr::Var { name, .. } => Expr::Path {
                        segments: vec![name, segment],
                        span: SourceSpan::new(start, end),
                    },
                    Expr::Path { mut segments, .. } => {
                        segments.push(segment);
                        Expr::Path {
                            segments,
                            span: SourceSpan::new(start, end),
                        }
                    }
                    _ => {
                        return Err(self.error_expected("a path segment"));
                    }
                };
            } else if self.at_keyword(Keyword::As) {
                self.advance();
                let ty = self.parse_type()?;
                let end = self.prev_end();
                let start = expr.span().start.clone();
                expr = Expr::Cast {
                    expr: Box::new(expr),
                    ty: Box::new(ty),
                    span: SourceSpan::new(start, end),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    
    fn parse_call_args(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        if !self.at_symbol(Symbol::RightParen) {
            loop {
                args.push(self.parse_expr()?);
                if !self.eat_symbol(Symbol::Comma) {
                    break;
                }
            }
        }
        self.expect_symbol(Symbol::RightParen, "`)`")?;
        Ok(args)
    }

    fn parse_primary(&mut self, no_struct: bool) -> Result<Expr, ParseError> {
        let start = self.peek_token().span.start.clone();
        match self.peek_kind() {
            TokenKind::Integer(lit) => {
                let value = lit.value;
                self.advance();
                Ok(Expr::Literal(
                    LiteralExpr::Integer(value),
                    self.span_from(start),
                ))
            }
            TokenKind::Float(f) => {
                let value = f.value;
                self.advance();
                Ok(Expr::Literal(
                    LiteralExpr::Float(value),
                    self.span_from(start),
                ))
            }
            TokenKind::Char(c) => {
                let c = *c;
                self.advance();
                Ok(Expr::Literal(LiteralExpr::Char(c), self.span_from(start)))
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Expr::Literal(
                    LiteralExpr::String(s),
                    self.span_from(start),
                ))
            }
            TokenKind::Keyword(Keyword::True) => {
                self.advance();
                Ok(Expr::Literal(
                    LiteralExpr::Bool(true),
                    self.span_from(start),
                ))
            }
            TokenKind::Keyword(Keyword::False) => {
                self.advance();
                Ok(Expr::Literal(
                    LiteralExpr::Bool(false),
                    self.span_from(start),
                ))
            }
            TokenKind::Keyword(Keyword::Null) => {
                self.advance();
                Ok(Expr::Literal(
                    LiteralExpr::Null,
                    self.span_from(start),
                ))
            }
            TokenKind::Symbol(Symbol::LeftBracket) => {
                self.parse_array_expr(start)
            }
            TokenKind::Keyword(Keyword::SelfKw) => {
                self.advance();
                Ok(Expr::Var {
                    name: "self".to_string(),
                    span: self.span_from(start),
                })
            }
            TokenKind::Keyword(Keyword::SelfType) => {
                self.advance();
                Ok(Expr::Var {
                    name: "Self".to_string(),
                    span: self.span_from(start),
                })
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                if name == "sizeOf" && self.at_symbol(Symbol::LeftBracket) {
                    
                    
                    self.advance(); 
                    let ty = self.parse_type()?;
                    self.expect_symbol(Symbol::RightBracket, "`]`")?;
                    if self.at_symbol(Symbol::LeftParen) {
                        self.advance();
                        self.expect_symbol(Symbol::RightParen, "`)`")?;
                    }
                    Ok(Expr::SizeOf {
                        ty: Box::new(ty),
                        span: self.span_from(start),
                    })
                } else if !no_struct && self.at_symbol(Symbol::LeftBrace) {
                    self.parse_struct_literal(name, Vec::new(), start)
                } else if !no_struct && self.at_symbol(Symbol::LeftBracket) && self.struct_literal_lookahead() {
                    
                    
                    
                    let args = self.parse_type_args()?;
                    self.expect_symbol(Symbol::LeftBrace, "`{`")?;
                    self.parse_struct_literal_fields(name, args, start)
                } else {
                    Ok(Expr::Var {
                        name,
                        span: self.span_from(start),
                    })
                }
            }
            TokenKind::Keyword(Keyword::Match) => self.parse_match(),
            TokenKind::Keyword(Keyword::Return) => self.parse_return_expr(),
            TokenKind::Symbol(Symbol::LeftParen) => {
                self.advance();
                let inner = self.parse_expr()?;
                self.expect_symbol(Symbol::RightParen, "`)`")?;
                Ok(inner)
            }
            _ => Err(self.error_expected("an expression")),
        }
    }

    
    
    
    fn struct_literal_lookahead(&self) -> bool {
        let mut pos = self.pos;
        
        if pos >= self.tokens.len() || !matches!(self.tokens[pos].kind, TokenKind::Symbol(Symbol::LeftBracket)) {
            return false;
        }
        pos += 1;
        
        let mut depth = 1usize;
        while pos < self.tokens.len() {
            match &self.tokens[pos].kind {
                TokenKind::Symbol(Symbol::LeftBracket) => depth += 1,
                TokenKind::Symbol(Symbol::RightBracket) => {
                    depth -= 1;
                    if depth == 0 {
                        pos += 1;
                        break;
                    }
                }
                TokenKind::Eof => return false,
                _ => {}
            }
            pos += 1;
        }
        if depth != 0 {
            return false;
        }
        pos < self.tokens.len()
            && matches!(self.tokens[pos].kind, TokenKind::Symbol(Symbol::LeftBrace))
    }

    
    fn parse_array_expr(&mut self, start: SourceLocation) -> Result<Expr, ParseError> {
        self.advance(); 
        let first = self.parse_expr()?;

        if self.at_symbol(Symbol::Semicolon) {
            
            self.advance(); 
            let count = self.parse_expr()?;
            self.expect_symbol(Symbol::RightBracket, "`]`")?;
            return Ok(Expr::ArrayRepeat {
                value: Box::new(first),
                count: Box::new(count),
                span: self.span_from(start),
            });
        }

        
        let mut elements = vec![first];
        while self.eat_symbol(Symbol::Comma) {
            if self.at_symbol(Symbol::RightBracket) {
                break; 
            }
            elements.push(self.parse_expr()?);
        }
        self.expect_symbol(Symbol::RightBracket, "`]`")?;
        Ok(Expr::ArrayLiteral {
            elements,
            span: self.span_from(start),
        })
    }

    fn parse_struct_literal(
        &mut self,
        name: String,
        args: Vec<Type>,
        start: SourceLocation,
    ) -> Result<Expr, ParseError> {
        self.expect_symbol(Symbol::LeftBrace, "`{`")?;
        self.parse_struct_literal_fields(name, args, start)
    }

    fn parse_struct_literal_fields(
        &mut self,
        name: String,
        args: Vec<Type>,
        start: SourceLocation,
    ) -> Result<Expr, ParseError> {
        let mut fields = Vec::new();
        if !self.at_symbol(Symbol::RightBrace) {
            loop {
                let field = self.expect_identifier("a field name")?;
                self.expect_symbol(Symbol::Colon, "`:`")?;
                let value = self.parse_expr()?;
                fields.push((field, value));
                if !self.eat_symbol(Symbol::Comma) {
                    break;
                }
            }
        }
        self.expect_symbol(Symbol::RightBrace, "`}`")?;
        Ok(Expr::StructLiteral {
            name,
            args,
            fields,
            span: self.span_from(start),
        })
    }

    fn parse_match(&mut self) -> Result<Expr, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::Match, "`match`")?;
        let scrutinee = self.parse_expr_no_struct()?;
        self.expect_symbol(Symbol::LeftBrace, "`{`")?;
        let mut arms = Vec::new();
        while !self.at_symbol(Symbol::RightBrace) {
            if self.at_eof() {
                return Err(self.error_expected("a match arm or `}`"));
            }
            let arm = self.parse_match_arm()?;
            let block_terminated = matches!(arm.body, MatchArmBody::Block(_));
            arms.push(arm);
            if block_terminated {
                
                self.eat_symbol(Symbol::Comma);
            } else if !self.eat_symbol(Symbol::Comma) && !self.at_symbol(Symbol::RightBrace) {
                return Err(self.error_expected("`,` or `}`"));
            }
        }
        self.expect_symbol(Symbol::RightBrace, "`}`")?;
        Ok(Expr::Match {
            scrutinee: Box::new(scrutinee),
            arms,
            span: self.span_from(start),
        })
    }

    fn parse_match_arm(&mut self) -> Result<MatchArm, ParseError> {
        let start = self.peek_token().span.start.clone();
        let pattern = self.parse_pattern()?;
        self.expect_symbol(Symbol::FatArrow, "`=>`")?;
        let body = if self.at_symbol(Symbol::LeftBrace) {
            MatchArmBody::Block(self.parse_block()?)
        } else {
            MatchArmBody::Expr(self.parse_expr()?)
        };
        Ok(MatchArm {
            pattern,
            body,
            span: self.span_from(start),
        })
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.peek_kind() {
            TokenKind::Symbol(Symbol::Underscore) => {
                self.advance();
                Ok(Pattern::Wildcard)
            }
            TokenKind::Keyword(Keyword::True) => {
                self.advance();
                Ok(Pattern::Literal(LiteralExpr::Bool(true)))
            }
            TokenKind::Keyword(Keyword::False) => {
                self.advance();
                Ok(Pattern::Literal(LiteralExpr::Bool(false)))
            }
            TokenKind::Integer(lit) => {
                let value = lit.value;
                self.advance();
                Ok(Pattern::Literal(LiteralExpr::Integer(value)))
            }
            TokenKind::Float(f) => {
                let value = f.value;
                self.advance();
                Ok(Pattern::Literal(LiteralExpr::Float(value)))
            }
            TokenKind::Char(c) => {
                let c = *c;
                self.advance();
                Ok(Pattern::Literal(LiteralExpr::Char(c)))
            }
            TokenKind::String(s) => {
                let s = s.clone();
                self.advance();
                Ok(Pattern::Literal(LiteralExpr::String(s)))
            }
            TokenKind::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(Pattern::Binding(name))
            }
            _ => Err(self.error_expected("a pattern")),
        }
    }

    fn parse_return_expr(&mut self) -> Result<Expr, ParseError> {
        let start = self.peek_token().span.start.clone();
        self.expect_keyword(Keyword::Return, "`return`")?;
        let value = if matches!(
            self.peek_kind(),
            TokenKind::Symbol(Symbol::Semicolon)
                | TokenKind::Symbol(Symbol::Comma)
                | TokenKind::Symbol(Symbol::RightBrace)
                | TokenKind::Symbol(Symbol::RightParen)
                | TokenKind::Eof
        ) {
            None
        } else {
            Some(Box::new(self.parse_expr()?))
        };
        Ok(Expr::Return {
            value,
            span: self.span_from(start),
        })
    }

    
    
    

    
    fn cur_binary_op(&self) -> Option<(BinaryOp, u8)> {
        let (op, prec) = match self.peek_kind() {
            TokenKind::Symbol(Symbol::OrOr) => (BinaryOp::OrOr, 1),
            TokenKind::Symbol(Symbol::AndAnd) => (BinaryOp::AndAnd, 2),
            TokenKind::Symbol(Symbol::Pipe) => (BinaryOp::BitOr, 3),
            TokenKind::Symbol(Symbol::Caret) => (BinaryOp::BitXor, 4),
            TokenKind::Symbol(Symbol::Amp) => (BinaryOp::BitAnd, 5),
            TokenKind::Symbol(Symbol::EqEq) => (BinaryOp::Eq, 6),
            TokenKind::Symbol(Symbol::BangEq) => (BinaryOp::NotEq, 6),
            TokenKind::Symbol(Symbol::Lt) => (BinaryOp::Lt, 7),
            TokenKind::Symbol(Symbol::Gt) => (BinaryOp::Gt, 7),
            TokenKind::Symbol(Symbol::LtEq) => (BinaryOp::LtEq, 7),
            TokenKind::Symbol(Symbol::GtEq) => (BinaryOp::GtEq, 7),
            TokenKind::Symbol(Symbol::Shl) => (BinaryOp::Shl, 8),
            TokenKind::Symbol(Symbol::Shr) => (BinaryOp::Shr, 8),
            TokenKind::Symbol(Symbol::Plus) => (BinaryOp::Add, 9),
            TokenKind::Symbol(Symbol::Minus) => (BinaryOp::Sub, 9),
            TokenKind::Symbol(Symbol::Star) => (BinaryOp::Mul, 10),
            TokenKind::Symbol(Symbol::Slash) => (BinaryOp::Div, 10),
            TokenKind::Symbol(Symbol::Percent) => (BinaryOp::Mod, 10),
            _ => return None,
        };
        Some((op, prec))
    }

    
    fn cur_assign_op(&self) -> Option<AssignOp> {
        match self.peek_kind() {
            TokenKind::Symbol(Symbol::Eq) => Some(AssignOp::Assign),
            TokenKind::Symbol(Symbol::PlusEq) => Some(AssignOp::Add),
            TokenKind::Symbol(Symbol::MinusEq) => Some(AssignOp::Sub),
            TokenKind::Symbol(Symbol::StarEq) => Some(AssignOp::Mul),
            TokenKind::Symbol(Symbol::SlashEq) => Some(AssignOp::Div),
            TokenKind::Symbol(Symbol::PercentEq) => Some(AssignOp::Mod),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::lex;

    
    fn parse(src: &str) -> Program {
        let tokens = lex(src, "test.cp").expect("lex should succeed");
        parse_tokens(tokens).unwrap_or_else(|e| panic!("unexpected parse error: {e}"))
    }

    
    fn parse_err(src: &str) -> ParseError {
        let tokens = lex(src, "test.cp").expect("lex should succeed");
        match parse_tokens(tokens) {
            Ok(_) => panic!("expected a parse error for {src:?}, but parsing succeeded"),
            Err(e) => e,
        }
    }

    
    fn only_fn(src: &str) -> Function {
        let prog = parse(src);
        assert_eq!(prog.items.len(), 1, "expected exactly one item");
        match &prog.items[0] {
            Item::Function(f) => f.clone(),
            other => panic!("expected a function item, got {other:?}"),
        }
    }

    
    fn fn_first_stmt(src: &str) -> Stmt {
        let f = only_fn(src);
        f.body.stmts.into_iter().next().expect("expected a statement")
    }

    fn expect_int(expr: &Expr, value: i128) {
        assert!(matches!(expr, Expr::Literal(LiteralExpr::Integer(v), _) if *v == value));
    }

    fn expect_var(expr: &Expr, name: &str) {
        assert!(matches!(expr, Expr::Var { name: n, .. } if n == name));
    }

    fn expect_binop(expr: &Expr, op: BinaryOp) -> (&Expr, &Expr) {
        match expr {
            Expr::Binary { op: o, lhs, rhs, .. } if *o == op => (lhs, rhs),
            other => panic!("expected binary op {op:?}, got {other:?}"),
        }
    }

    
    
    

    #[test]
    fn parses_empty_program() {
        assert!(parse("").items.is_empty());
        assert!(parse("// just a comment").items.is_empty());
    }

    #[test]
    fn parses_import_paths() {
        let prog = parse("import std.io;");
        assert_eq!(prog.items.len(), 1);
        match &prog.items[0] {
            Item::Import(imp) => assert_eq!(imp.path, vec!["std".to_string(), "io".to_string()]),
            other => panic!("expected import, got {other:?}"),
        }

        let prog = parse("import math;");
        match &prog.items[0] {
            Item::Import(imp) => assert_eq!(imp.path, vec!["math".to_string()]),
            other => panic!("expected import, got {other:?}"),
        }
    }

    #[test]
    fn parses_basic_function() {
        let f = only_fn("fn add(a: i32, b: i32) -> i32 { return a + b; }");
        assert_eq!(f.name, "add");
        assert!(f.generics.is_empty());
        assert_eq!(f.params.len(), 2);
        assert_eq!(f.params[0].name, "a");
        assert!(matches!(f.params[0].ty, Type::Primitive(PrimitiveType::I32)));
        assert_eq!(f.params[1].name, "b");
        assert!(matches!(f.return_ty, Some(Type::Primitive(PrimitiveType::I32))));
    }

    #[test]
    fn parses_function_without_return_type() {
        let f = only_fn("fn f() { }");
        assert_eq!(f.name, "f");
        assert!(f.params.is_empty());
        assert!(f.return_ty.is_none());
        assert!(f.body.stmts.is_empty());
    }

    #[test]
    fn parses_generic_function_with_pointer_params() {
        let f = only_fn("fn swap[T](a: *mut T, b: *mut T) {}");
        assert_eq!(f.generics, vec!["T".to_string()]);
        for param in &f.params {
            match &param.ty {
                Type::Pointer { pointee, mutability, .. } => {
                    assert_eq!(*mutability, PointerMutability::Mut);
                    assert!(matches!(
                        &**pointee,
                        Type::Named { name, .. } if name == "T"
                    ));
                }
                other => panic!("expected pointer type, got {other:?}"),
            }
        }
    }

    #[test]
    fn parses_const_pointer_types() {
        let f = only_fn("fn f(p: *const i32) -> *const i32 { return p; }");
        match &f.params[0].ty {
            Type::Pointer { pointee, mutability, .. } => {
                assert_eq!(*mutability, PointerMutability::Const);
                assert!(matches!(&**pointee, Type::Primitive(PrimitiveType::I32)));
            }
            other => panic!("expected pointer, got {other:?}"),
        }
    }

    #[test]
    fn parses_class_with_fields() {
        let prog = parse("class Vector2 { x: f64, y: f64 }");
        match &prog.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.name, "Vector2");
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0].name, "x");
                assert!(matches!(s.fields[0].ty, Type::Primitive(PrimitiveType::F64)));
                assert_eq!(s.fields[1].name, "y");
            }
            other => panic!("expected struct/class, got {other:?}"),
        }
    }

    #[test]
    fn parses_class_with_methods() {
        let prog = parse(
            "class Vector2 { x: f64, fn new(x: f64) -> Vector2 { return Vector2 { x: x }; } }",
        );
        match &prog.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.fields.len(), 1);
                assert_eq!(s.methods.len(), 1);
                assert_eq!(s.methods[0].name, "new");
            }
            other => panic!("expected class with methods, got {other:?}"),
        }
    }

    #[test]
    fn parses_struct_with_fields() {
        let prog = parse("struct Vector2 { x: f64, y: f64 }");
        match &prog.items[0] {
            Item::Struct(s) => {
                assert_eq!(s.name, "Vector2");
                assert_eq!(s.fields.len(), 2);
                assert_eq!(s.fields[0].name, "x");
                assert!(matches!(s.fields[0].ty, Type::Primitive(PrimitiveType::F64)));
                assert_eq!(s.fields[1].name, "y");
            }
            other => panic!("expected struct, got {other:?}"),
        }
    }

    #[test]
    fn parses_struct_with_trailing_comma() {
        let prog = parse("struct P { x: i32, y: i32, }");
        match &prog.items[0] {
            Item::Struct(s) => assert_eq!(s.fields.len(), 2),
            other => panic!("expected struct, got {other:?}"),
        }
    }

    #[test]
    fn parses_impl_block_with_methods() {
        let prog = parse(
            "impl Vector2 { fn new(x: f64, y: f64) -> Vector2 { return Vector2 { x: x, y: y }; } }",
        );
        match &prog.items[0] {
            Item::Impl(blk) => {
                assert_eq!(blk.type_name, "Vector2");
                assert_eq!(blk.methods.len(), 1);
                assert_eq!(blk.methods[0].name, "new");
                assert_eq!(blk.methods[0].params.len(), 2);
            }
            other => panic!("expected impl, got {other:?}"),
        }
    }

    #[test]
    fn parses_self_parameter_in_methods() {
        let prog = parse("impl Vector2 { fn len(self) -> i32 { return 2; } }");
        match &prog.items[0] {
            Item::Impl(blk) => {
                assert_eq!(blk.methods[0].params[0].name, "self");
                assert!(matches!(
                    blk.methods[0].params[0].ty,
                    Type::Named { ref name, .. } if name == "Self"
                ));
            }
            other => panic!("expected impl, got {other:?}"),
        }
    }

    #[test]
    fn parses_self_type_in_type_positions() {
        let prog = parse(
            "impl Vector2 { fn clone(self) -> Self { return self; } fn ptr(self) -> *const Self { return &self; } }",
        );
        match &prog.items[0] {
            Item::Impl(blk) => {
                let clone_ret = blk.methods[0].return_ty.as_ref().expect("return type");
                assert!(matches!(
                    clone_ret,
                    Type::Named { name, .. } if name.as_str() == "Self"
                ));
                let ptr_ret = blk.methods[1].return_ty.as_ref().expect("return type");
                match ptr_ret {
                    Type::Pointer { pointee, mutability, .. } => {
                        assert_eq!(*mutability, PointerMutability::Const);
                        assert!(matches!(
                            &**pointee,
                            Type::Named { name, .. } if name.as_str() == "Self"
                        ));
                    }
                    other => panic!("expected pointer, got {other:?}"),
                }
            }
            other => panic!("expected impl, got {other:?}"),
        }
    }

    #[test]
    fn parses_multiple_items_in_order() {
        let prog = parse("import std.io; struct A { x: i32 } fn f() {}");
        assert_eq!(prog.items.len(), 3);
        assert!(matches!(prog.items[0], Item::Import(_)));
        assert!(matches!(prog.items[1], Item::Struct(_)));
        assert!(matches!(prog.items[2], Item::Function(_)));
    }

    
    
    

    #[test]
    #[allow(clippy::approx_constant)] 
    fn parses_let_statements() {
        let stmt = fn_first_stmt("fn f() { let x: i32 = 10; }");
        let Stmt::Let(let_stmt) = &stmt else {
            panic!("expected let, got {stmt:?}")
        };
        assert_eq!(let_stmt.name, "x");
        assert!(!let_stmt.mutable);
        assert!(matches!(let_stmt.ty, Some(Type::Primitive(PrimitiveType::I32))));
        expect_int(let_stmt.init.as_ref().unwrap(), 10);

        let stmt = fn_first_stmt("fn f() { let mut y: f64 = 3.14; }");
        let Stmt::Let(let_stmt) = &stmt else {
            panic!("expected let, got {stmt:?}")
        };
        assert!(let_stmt.mutable);
        assert!(matches!(let_stmt.ty, Some(Type::Primitive(PrimitiveType::F64))));
        assert!(matches!(
            let_stmt.init,
            Some(Expr::Literal(LiteralExpr::Float(3.14), _))
        ));
    }

    #[test]
    fn parses_let_without_type_or_init() {
        let stmt = fn_first_stmt("fn f() { let mut x; }");
        let Stmt::Let(let_stmt) = &stmt else {
            panic!("expected let, got {stmt:?}")
        };
        assert_eq!(let_stmt.name, "x");
        assert!(let_stmt.mutable);
        assert!(let_stmt.ty.is_none());
        assert!(let_stmt.init.is_none());

        let stmt = fn_first_stmt("fn f() { let y = 1; }");
        let Stmt::Let(let_stmt) = &stmt else {
            panic!("expected let, got {stmt:?}")
        };
        assert!(let_stmt.ty.is_none());
        expect_int(let_stmt.init.as_ref().unwrap(), 1);
    }

    #[test]
    fn parses_return_statements() {
        let stmt = fn_first_stmt("fn f() { return 42; }");
        let Stmt::Expr(Expr::Return { value, .. }) = &stmt else {
            panic!("expected return, got {stmt:?}")
        };
        expect_int(value.as_ref().unwrap(), 42);

        let stmt = fn_first_stmt("fn f() { return; }");
        let Stmt::Expr(Expr::Return { value, .. }) = &stmt else {
            panic!("expected return, got {stmt:?}")
        };
        assert!(value.is_none());
    }

    #[test]
    fn parses_nested_blocks() {
        let stmt = fn_first_stmt("fn f() { { let a = 1; } }");
        let Stmt::Block(block) = &stmt else {
            panic!("expected block, got {stmt:?}")
        };
        assert_eq!(block.stmts.len(), 1);
        assert!(matches!(block.stmts[0], Stmt::Let(_)));
    }

    #[test]
    fn parses_multiple_statements() {
        let f = only_fn("fn f() { let a = 1; let b = 2; return a + b; }");
        assert_eq!(f.body.stmts.len(), 3);
    }

    
    
    

    #[test]
    fn parses_if_else() {
        let stmt = fn_first_stmt("fn f() { if x > 5 { return 1; } else { return 0; } }");
        let Stmt::If(if_stmt) = &stmt else {
            panic!("expected if, got {stmt:?}")
        };
        let (lhs, rhs) = expect_binop(&if_stmt.condition, BinaryOp::Gt);
        expect_var(lhs, "x");
        expect_int(rhs, 5);
        assert_eq!(if_stmt.then_block.stmts.len(), 1);
        match &if_stmt.else_branch {
            Some(ElseBranch::Block(b)) => assert_eq!(b.stmts.len(), 1),
            other => panic!("expected else block, got {other:?}"),
        }
    }

    #[test]
    fn parses_if_without_else() {
        let stmt = fn_first_stmt("fn f() { if ok { } }");
        let Stmt::If(if_stmt) = &stmt else {
            panic!("expected if, got {stmt:?}")
        };
        expect_var(&if_stmt.condition, "ok");
        assert!(if_stmt.else_branch.is_none());
    }

    #[test]
    fn parses_else_if_chain() {
        let src = "fn f() { if a { } else if b { } else if c { } else { } }";
        let stmt = fn_first_stmt(src);
        let Stmt::If(first) = &stmt else {
            panic!("expected if, got {stmt:?}")
        };
        let Some(ElseBranch::If(second)) = &first.else_branch else {
            panic!("expected else-if")
        };
        let Some(ElseBranch::If(third)) = &second.else_branch else {
            panic!("expected second else-if")
        };
        let Some(ElseBranch::Block(_)) = &third.else_branch else {
            panic!("expected final else block")
        };
        expect_var(&second.condition, "b");
        expect_var(&third.condition, "c");
    }

    #[test]
    fn parses_while_loop() {
        let stmt = fn_first_stmt("fn f() { while x < 10 { x = x + 1; } }");
        let Stmt::While(while_stmt) = &stmt else {
            panic!("expected while, got {stmt:?}")
        };
        let (lhs, rhs) = expect_binop(&while_stmt.condition, BinaryOp::Lt);
        expect_var(lhs, "x");
        expect_int(rhs, 10);
        assert_eq!(while_stmt.body.stmts.len(), 1);
    }

    #[test]
    fn parses_for_loop() {
        let stmt = fn_first_stmt("fn f() { for i in items { println(i); } }");
        let Stmt::For(for_stmt) = &stmt else {
            panic!("expected for, got {stmt:?}")
        };
        assert_eq!(for_stmt.variable, "i");
        expect_var(&for_stmt.iterable, "items");
        assert_eq!(for_stmt.body.stmts.len(), 1);
    }

    #[test]
    fn for_loop_does_not_misparse_body_as_struct_literal() {
        
        let stmt = fn_first_stmt("fn f() { for x in v { g(x); } }");
        let Stmt::For(for_stmt) = &stmt else {
            panic!("expected for, got {stmt:?}")
        };
        expect_var(&for_stmt.iterable, "v");
        assert_eq!(for_stmt.body.stmts.len(), 1);
    }

    
    
    

    #[test]
    fn parses_literals() {
        let stmt = fn_first_stmt("fn f() { g(true, 42, 3.5, 'a', \"hi\"); }");
        let Stmt::Expr(Expr::Call { args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        assert!(matches!(&args[0], Expr::Literal(LiteralExpr::Bool(true), _)));
        expect_int(&args[1], 42);
        assert!(matches!(&args[2], Expr::Literal(LiteralExpr::Float(3.5), _)));
        assert!(matches!(&args[3], Expr::Literal(LiteralExpr::Char('a'), _)));
        assert!(matches!(&args[4], Expr::Literal(LiteralExpr::String(s), _) if s == "hi"));
    }

    #[test]
    fn multiplicative_binds_tighter_than_additive() {
        let stmt = fn_first_stmt("fn f() { g(1 + 2 * 3); }");
        let Stmt::Expr(Expr::Call { args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        let (lhs, rhs) = expect_binop(&args[0], BinaryOp::Add);
        expect_int(lhs, 1);
        let (_, _) = expect_binop(rhs, BinaryOp::Mul);
    }

    #[test]
    fn binary_operators_are_left_associative() {
        let stmt = fn_first_stmt("fn f() { g(10 - 3 - 2); }");
        let Stmt::Expr(Expr::Call { args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        let (lhs, _rhs) = expect_binop(&args[0], BinaryOp::Sub);
        let (_, _) = expect_binop(lhs, BinaryOp::Sub);
    }

    #[test]
    fn logical_operators_precedence() {
        let stmt = fn_first_stmt("fn f() { g(a || b && c); }");
        let Stmt::Expr(Expr::Call { args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        let (lhs, rhs) = expect_binop(&args[0], BinaryOp::OrOr);
        expect_var(lhs, "a");
        let (_, _) = expect_binop(rhs, BinaryOp::AndAnd);
    }

    #[test]
    fn comparison_precedence() {
        let stmt = fn_first_stmt("fn f() { g(a < b == c); }");
        let Stmt::Expr(Expr::Call { args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        let (lhs, rhs) = expect_binop(&args[0], BinaryOp::Eq);
        let (_, _) = expect_binop(lhs, BinaryOp::Lt);
        expect_var(rhs, "c");
    }

    #[test]
    fn shift_binds_tighter_than_comparison() {
        let stmt = fn_first_stmt("fn f() { g(a << b < c); }");
        let Stmt::Expr(Expr::Call { args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        let (lhs, rhs) = expect_binop(&args[0], BinaryOp::Lt);
        let (_, _) = expect_binop(lhs, BinaryOp::Shl);
        expect_var(rhs, "c");
    }

    #[test]
    fn bitwise_and_binds_tighter_than_or() {
        let stmt = fn_first_stmt("fn f() { g(a & b | c); }");
        let Stmt::Expr(Expr::Call { args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        let (lhs, rhs) = expect_binop(&args[0], BinaryOp::BitOr);
        let (_, _) = expect_binop(lhs, BinaryOp::BitAnd);
        expect_var(rhs, "c");
    }

    #[test]
    fn unary_minus_binds_looser_than_multiplication() {
        let stmt = fn_first_stmt("fn f() { g(-a * b); }");
        let Stmt::Expr(Expr::Call { args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        let (lhs, rhs) = expect_binop(&args[0], BinaryOp::Mul);
        expect_var(rhs, "b");
        assert!(matches!(
            lhs,
            Expr::Unary { op: UnaryOp::Neg, .. }
        ));
    }

    #[test]
    fn parses_all_unary_operators() {
        let check = |src: &str, want: UnaryOp| {
            let stmt = fn_first_stmt(src);
            let Stmt::Expr(Expr::Call { args, .. }) = &stmt else {
                panic!("expected call, got {stmt:?}")
            };
            assert!(matches!(
                &args[0],
                Expr::Unary { op, .. } if *op == want
            ));
        };
        check("fn f() { g(-x); }", UnaryOp::Neg);
        check("fn f() { g(!x); }", UnaryOp::Not);
        check("fn f() { g(~x); }", UnaryOp::BitNot);
        check("fn f() { g(*p); }", UnaryOp::Deref);
        check("fn f() { g(&x); }", UnaryOp::AddrOf);
        check("fn f() { g(&mut x); }", UnaryOp::AddrOfMut);
    }

    #[test]
    fn unary_operators_stack() {
        let stmt = fn_first_stmt("fn f() { g(-*p); }");
        let Stmt::Expr(Expr::Call { args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        assert!(matches!(
            &args[0],
            Expr::Unary { op: UnaryOp::Neg, operand, .. } if matches!(
                &**operand,
                Expr::Unary { op: UnaryOp::Deref, .. }
            )
        ));
    }

    #[test]
    fn parses_parenthesized_expression() {
        let stmt = fn_first_stmt("fn f() { g((1 + 2) * 3); }");
        let Stmt::Expr(Expr::Call { args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        let (lhs, rhs) = expect_binop(&args[0], BinaryOp::Mul);
        expect_int(rhs, 3);
        let (_, _) = expect_binop(lhs, BinaryOp::Add);
    }

    #[test]
    fn parses_function_calls() {
        let stmt = fn_first_stmt("fn f() { foo(1, 2); }");
        let Stmt::Expr(Expr::Call { callee, args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        expect_var(callee, "foo");
        assert_eq!(args.len(), 2);

        let stmt = fn_first_stmt("fn f() { bar(); }");
        let Stmt::Expr(Expr::Call { args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        assert!(args.is_empty());
    }

    #[test]
    fn parses_method_calls_and_field_access() {
        let stmt = fn_first_stmt("fn f() { v.len(); }");
        let Stmt::Expr(Expr::MethodCall { receiver, method, args, .. }) = &stmt else {
            panic!("expected method call, got {stmt:?}")
        };
        expect_var(receiver, "v");
        assert_eq!(method, "len");
        assert!(args.is_empty());

        let stmt = fn_first_stmt("fn f() { v.x; }");
        let Stmt::Expr(Expr::FieldAccess { base, field, .. }) = &stmt else {
            panic!("expected field access, got {stmt:?}")
        };
        expect_var(base, "v");
        assert_eq!(field, "x");

        let stmt = fn_first_stmt("fn f() { p.x.y; }");
        let Stmt::Expr(Expr::FieldAccess { base, field, .. }) = &stmt else {
            panic!("expected field access, got {stmt:?}")
        };
        assert_eq!(field, "y");
        assert!(matches!(
            &**base,
            Expr::FieldAccess { field, .. } if field == "x"
        ));
    }

    #[test]
    fn parses_index_expressions() {
        let stmt = fn_first_stmt("fn f() { a[i]; }");
        let Stmt::Expr(Expr::Index { base, index, .. }) = &stmt else {
            panic!("expected index, got {stmt:?}")
        };
        expect_var(base, "a");
        expect_var(index, "i");

        let stmt = fn_first_stmt("fn f() { m[k][j]; }");
        let Stmt::Expr(Expr::Index { base, index, .. }) = &stmt else {
            panic!("expected index, got {stmt:?}")
        };
        expect_var(index, "j");
        assert!(matches!(&**base, Expr::Index { .. }));
    }

    #[test]
    fn parses_path_calls() {
        let stmt = fn_first_stmt("fn f() { Vector2::new(1.0, 2.0); }");
        let Stmt::Expr(Expr::Call { callee, args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        assert!(matches!(
            &**callee,
            Expr::Path { segments, .. } if *segments == vec!["Vector2".to_string(), "new".to_string()]
        ));
        assert_eq!(args.len(), 2);
    }

    #[test]
    fn parses_struct_literals() {
        let stmt = fn_first_stmt("fn f() { let v = Vector2 { x: 1.0, y: 2.0 }; }");
        let Stmt::Let(let_stmt) = &stmt else {
            panic!("expected let, got {stmt:?}")
        };
        match let_stmt.init.as_ref().unwrap() {
            Expr::StructLiteral { name, fields, .. } => {
                assert_eq!(name, "Vector2");
                assert_eq!(fields.len(), 2);
                assert_eq!(fields[0].0, "x");
                assert!(matches!(&fields[0].1, Expr::Literal(LiteralExpr::Float(1.0), _)));
                assert_eq!(fields[1].0, "y");
                assert!(matches!(&fields[1].1, Expr::Literal(LiteralExpr::Float(2.0), _)));
            }
            other => panic!("expected struct literal, got {other:?}"),
        }
    }

    #[test]
    fn parses_cast_expressions() {
        let stmt = fn_first_stmt("fn f() { g(x as f64); }");
        let Stmt::Expr(Expr::Call { args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        match &args[0] {
            Expr::Cast { expr, ty, .. } => {
                expect_var(expr, "x");
                assert!(matches!(&**ty, Type::Primitive(PrimitiveType::F64)));
            }
            other => panic!("expected cast, got {other:?}"),
        }
    }

    #[test]
    fn parses_assignments() {
        let stmt = fn_first_stmt("fn f() { x = 5; }");
        let Stmt::Expr(Expr::Assign { target, op, value, .. }) = &stmt else {
            panic!("expected assign, got {stmt:?}")
        };
        expect_var(target, "x");
        assert_eq!(*op, AssignOp::Assign);
        expect_int(value, 5);

        let stmt = fn_first_stmt("fn f() { x += 1; }");
        let Stmt::Expr(Expr::Assign { op, .. }) = &stmt else {
            panic!("expected assign, got {stmt:?}")
        };
        assert_eq!(*op, AssignOp::Add);

        let stmt = fn_first_stmt("fn f() { x -= 1; }");
        let Stmt::Expr(Expr::Assign { op, .. }) = &stmt else {
            panic!("expected assign, got {stmt:?}")
        };
        assert_eq!(*op, AssignOp::Sub);
    }

    #[test]
    fn assignments_are_right_associative() {
        let stmt = fn_first_stmt("fn f() { x = y = 3; }");
        let Stmt::Expr(Expr::Assign { value, .. }) = &stmt else {
            panic!("expected assign, got {stmt:?}")
        };
        assert!(matches!(&**value, Expr::Assign { .. }));
    }

    #[test]
    fn parses_generic_type_instantiations() {
        let f = only_fn("fn f(o: Option[i32], r: Result[i32, String]) { }");
        match &f.params[0].ty {
            Type::Named { name, args, .. } => {
                assert_eq!(name, "Option");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], Type::Primitive(PrimitiveType::I32)));
            }
            other => panic!("expected named type, got {other:?}"),
        }
        match &f.params[1].ty {
            Type::Named { name, args, .. } => {
                assert_eq!(name, "Result");
                assert_eq!(args.len(), 2);
            }
            other => panic!("expected named type, got {other:?}"),
        }
    }

    #[test]
    fn match_heads_do_not_consume_blocks() {
        
        let stmt = fn_first_stmt("fn f() { if v { g(); } }");
        let Stmt::If(if_stmt) = &stmt else {
            panic!("expected if, got {stmt:?}")
        };
        expect_var(&if_stmt.condition, "v");
        assert_eq!(if_stmt.then_block.stmts.len(), 1);

        let stmt = fn_first_stmt("fn f() { while w { g(); } }");
        let Stmt::While(while_stmt) = &stmt else {
            panic!("expected while, got {stmt:?}")
        };
        expect_var(&while_stmt.condition, "w");
    }

    
    
    

    #[test]
    fn parses_match_with_literal_and_wildcard_arms() {
        let stmt = fn_first_stmt("fn f() { match val { 1 => true, _ => false } }");
        let Stmt::Expr(Expr::Match { scrutinee, arms, .. }) = &stmt else {
            panic!("expected match, got {stmt:?}")
        };
        expect_var(scrutinee, "val");
        assert_eq!(arms.len(), 2);

        let arm1 = &arms[0];
        assert!(matches!(&arm1.pattern, Pattern::Literal(LiteralExpr::Integer(1))));
        assert!(matches!(
            &arm1.body,
            MatchArmBody::Expr(Expr::Literal(LiteralExpr::Bool(true), _))
        ));

        let arm2 = &arms[1];
        assert!(matches!(arm2.pattern, Pattern::Wildcard));
        assert!(matches!(
            &arm2.body,
            MatchArmBody::Expr(Expr::Literal(LiteralExpr::Bool(false), _))
        ));
    }

    #[test]
    fn parses_match_arm_with_return_and_block() {
        let src = "fn f() { match val { 42 => return 1, _ => { let x = 2; return x; } } }";
        let stmt = fn_first_stmt(src);
        let Stmt::Expr(Expr::Match { arms, .. }) = &stmt else {
            panic!("expected match, got {stmt:?}")
        };
        assert_eq!(arms.len(), 2);

        assert!(matches!(
            &arms[0].body,
            MatchArmBody::Expr(Expr::Return { value, .. }) if matches!(
                value,
                Some(box_expr) if matches!(**box_expr, Expr::Literal(LiteralExpr::Integer(1), _))
            )
        ));

        match &arms[1].body {
            MatchArmBody::Block(block) => assert_eq!(block.stmts.len(), 2),
            other => panic!("expected block body, got {other:?}"),
        }
    }

    #[test]
    fn parses_binding_patterns() {
        let stmt = fn_first_stmt("fn f() { match x { n => g(n) } }");
        let Stmt::Expr(Expr::Match { arms, .. }) = &stmt else {
            panic!("expected match, got {stmt:?}")
        };
        assert!(matches!(&arms[0].pattern, Pattern::Binding(name) if name == "n"));
    }

    #[test]
    fn parses_string_and_char_patterns() {
        let stmt = fn_first_stmt(r#"fn f() { match c { 'a' => 1, "x" => 2, _ => 3 } }"#);
        let Stmt::Expr(Expr::Match { arms, .. }) = &stmt else {
            panic!("expected match, got {stmt:?}")
        };
        assert!(matches!(&arms[0].pattern, Pattern::Literal(LiteralExpr::Char('a'))));
        assert!(matches!(&arms[1].pattern, Pattern::Literal(LiteralExpr::String(s)) if s == "x"));
    }

    #[test]
    fn parses_match_without_trailing_comma() {
        let src = "fn f() { match v { 1 => a, _ => b } }";
        let stmt = fn_first_stmt(src);
        let Stmt::Expr(Expr::Match { arms, .. }) = &stmt else {
            panic!("expected match, got {stmt:?}")
        };
        assert_eq!(arms.len(), 2);
    }

    
    
    

    #[test]
    fn tracks_item_spans() {
        let prog = parse("struct A { x: i32 }\nfn f() {}\n");
        assert_eq!(prog.items.len(), 2);
        let s = prog.items[0].span();
        assert_eq!(s.start.line, 1);
        assert_eq!(s.start.column, 1);
        let f = prog.items[1].span();
        assert_eq!(f.start.line, 2);
        assert_eq!(f.start.column, 1);
        assert_eq!(prog.span.start.line, 1);
        assert_eq!(prog.span.end.line, 2);
    }

    #[test]
    fn tracks_expression_spans() {
        let f = only_fn("fn f() { g(a + b); }");
        let Stmt::Expr(Expr::Call { span, .. }) = &f.body.stmts[0] else {
            panic!("expected call")
        };
        
        assert_eq!(span.start.column, 10);
    }

    
    
    

    #[test]
    fn error_on_missing_semicolon() {
        assert!(matches!(parse_err("fn f() { let x = 1 }"), ParseError::UnexpectedToken { .. }));
    }

    #[test]
    fn error_on_missing_closing_paren() {
        assert!(matches!(
            parse_err("fn f() { g(1, 2; }"),
            ParseError::UnexpectedToken { .. }
        ));
    }

    #[test]
    fn error_on_unterminated_block() {
        match parse_err("fn f() {") {
            ParseError::UnexpectedEof { expected, .. } => assert!(expected.contains("`}`")),
            other => panic!("expected unexpected-eof, got {other:?}"),
        }
    }

    #[test]
    fn error_on_unexpected_top_level_token() {
        match parse_err("garbage") {
            ParseError::UnexpectedToken { expected, .. } => {
                assert!(expected.contains("top-level item"))
            }
            other => panic!("expected unexpected-token, got {other:?}"),
        }
    }

    #[test]
    fn error_on_malformed_type() {
        assert!(matches!(
            parse_err("fn f() { let x: = 1; }"),
            ParseError::UnexpectedToken { .. }
        ));
    }

    #[test]
    fn error_on_bare_pointer_star() {
        assert!(matches!(
            parse_err("fn f(p: * i32) { }"),
            ParseError::UnexpectedToken { .. }
        ));
    }

    #[test]
    fn error_on_struct_literal_missing_fields() {
        assert!(matches!(
            parse_err("fn f() { Vector2 { x }; }"),
            ParseError::UnexpectedToken { .. }
        ));
    }

    #[test]
    fn error_identifies_expected_and_found() {
        let err = parse_err("fn f() { let x = ; }");
        match err {
            ParseError::UnexpectedToken { expected, found, .. } => {
                assert_eq!(expected, "an expression");
                assert_eq!(found, "`;`");
            }
            other => panic!("expected unexpected-token, got {other:?}"),
        }
    }

    #[test]
    fn error_on_return_outside_expression_is_ok() {
        
        let src = "fn f() -> i32 { match x { _ => return 5, } }";
        let f = only_fn(src);
        assert_eq!(f.body.stmts.len(), 1);
    }

    
    
    

    #[test]
    fn parses_range_for_loop() {
        let src = "fn f() { for i in 0..10 { println(i); } }";
        let stmt = fn_first_stmt(src);
        
        let Stmt::Block(block) = &stmt else {
            panic!("expected desugared block, got {stmt:?}")
        };
        
        assert_eq!(block.stmts.len(), 4);
        assert!(matches!(&block.stmts[3], Stmt::While(_)));
    }

    #[test]
    fn parses_inclusive_range_for_loop() {
        let src = "fn f() { for i in 0..=10 { println(i); } }";
        let stmt = fn_first_stmt(src);
        let Stmt::Block(block) = &stmt else {
            panic!("expected desugared block, got {stmt:?}")
        };
        assert_eq!(block.stmts.len(), 4);
        assert!(matches!(&block.stmts[3], Stmt::While(_)));
    }

    #[test]
    fn parses_array_repeat_literal() {
        let src = "fn f() { g([0; 10]); }";
        let stmt = fn_first_stmt(src);
        let Stmt::Expr(Expr::Call { args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        assert!(matches!(&args[0], Expr::ArrayRepeat { .. }));
    }

    #[test]
    fn parses_array_literal() {
        let src = "fn f() { g([1, 2, 3]); }";
        let stmt = fn_first_stmt(src);
        let Stmt::Expr(Expr::Call { args, .. }) = &stmt else {
            panic!("expected call, got {stmt:?}")
        };
        match &args[0] {
            Expr::ArrayLiteral { elements, .. } => {
                assert_eq!(elements.len(), 3);
            }
            other => panic!("expected array literal, got {other:?}"),
        }
    }

    #[test]
    fn parses_array_type() {
        let src = "fn f(a: [i32; 10]) { }";
        let f = only_fn(src);
        let Type::Array { element, length, .. } = &f.params[0].ty else {
            panic!("expected array type")
        };
        assert!(matches!(&**element, Type::Primitive(PrimitiveType::I32)));
        expect_int(length, 10);
    }

    #[test]
    fn parses_array_index() {
        let src = "fn f() { let mut a: [i32; 5] = [0; 5]; g(a[2]); }";
        let f = only_fn(src);
        assert_eq!(f.body.stmts.len(), 2);
        let Stmt::Expr(Expr::Call { args, .. }) = &f.body.stmts[1] else {
            panic!("expected call")
        };
        assert!(matches!(&args[0], Expr::Index { .. }));
    }
}

