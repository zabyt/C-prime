

















use std::collections::HashMap;

use crate::ast::*;
use crate::symbol::{CType, FuncSig, ScopeManager, StructInfo, VarInfo};
use crate::token::SourceSpan;


#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TypeError {
    #[error("{span}: undefined variable `{name}`")]
    UndefinedVariable { span: SourceSpan, name: String },

    #[error("{span}: undefined type `{name}`")]
    UndefinedType { span: SourceSpan, name: String },

    #[error("{span}: undefined function `{name}`")]
    UndefinedFunction { span: SourceSpan, name: String },

    #[error("{span}: undefined method `{name}` on type `{ty}`")]
    UndefinedMethod { span: SourceSpan, name: String, ty: String },

    #[error("{span}: `{name}` is not a field of `{ty}`")]
    UnknownField { span: SourceSpan, name: String, ty: String },

    #[error("{span}: type `{ty}` has no field `{field}`")]
    UnknownFieldAccess { span: SourceSpan, ty: String, field: String },

    #[error("{span}: field access on a non-struct type `{ty}`")]
    NotAStruct { span: SourceSpan, ty: String },

    #[error("{span}: member `{member}` of `{ty}` is private")]
    PrivateMemberAccess { span: SourceSpan, member: String, ty: String },

    #[error("{span}: `{method}` cannot override `{base}` because `{base}` does not declare a virtual `{method}`")]
    InvalidOverride { span: SourceSpan, method: String, base: String },

    #[error("{span}: `{name}` is final and cannot be overridden")]
    FinalOverride { span: SourceSpan, name: String },

    #[error("{span}: struct literal for `{name}` is missing field `{field}`")]
    MissingField { span: SourceSpan, name: String, field: String },

    #[error("{span}: duplicate field `{name}` in struct literal")]
    DuplicateFieldInLiteral { span: SourceSpan, name: String },

    #[error("{span}: duplicate parameter `{name}`")]
    DuplicateParam { span: SourceSpan, name: String },

    #[error("{span}: duplicate field `{name}` in struct `{ty}`")]
    DuplicateStructField { span: SourceSpan, name: String, ty: String },

    #[error("{span}: duplicate type parameter `{name}`")]
    DuplicateGenericParam { span: SourceSpan, name: String },

    #[error("{span}: `{name}` is defined more than once")]
    DuplicateName { span: SourceSpan, name: String },

    #[error("{span}: type mismatch: expected `{expected}`, found `{found}`")]
    TypeMismatch { span: SourceSpan, expected: String, found: String },

    #[error("{span}: `{name}` is already defined in this scope")]
    AlreadyDefined { span: SourceSpan, name: String },

    #[error("{span}: cannot assign to immutable variable `{name}`")]
    AssignToImmutable { span: SourceSpan, name: String },

    #[error("{span}: cannot assign to this expression")]
    InvalidAssignTarget { span: SourceSpan },

    #[error("{span}: cannot assign through a `*const` pointer")]
    AssignThroughConstPointer { span: SourceSpan },

    #[error("{span}: `if` condition must be `bool`, found `{found}`")]
    IfConditionNotBool { span: SourceSpan, found: String },

    #[error("{span}: `while` condition must be `bool`, found `{found}`")]
    WhileConditionNotBool { span: SourceSpan, found: String },

    #[error("{span}: expected {expected} argument(s), found {found}")]
    ArgCountMismatch { span: SourceSpan, expected: usize, found: usize },

    #[error("{span}: `{name}` is not callable")]
    NotCallable { span: SourceSpan, name: String },

    #[error("{span}: method `{name}` requires a receiver")]
    MethodRequiresReceiver { span: SourceSpan, name: String },

    #[error("{span}: return type mismatch: expected `{expected}`, found `{found}`")]
    ReturnMismatch { span: SourceSpan, expected: String, found: String },

    #[error("{span}: `return;` in a function with return type `{expected}` requires a value")]
    MissingReturnValue { span: SourceSpan, expected: String },

    #[error("{span}: `return` with a value in a `void` function")]
    VoidReturnValue { span: SourceSpan },

    #[error("{span}: `break` must appear in a switch or loop")]
    BreakOutsideLoopOrSwitch { span: SourceSpan },

    #[error("{span}: `continue` must appear in a loop")]
    ContinueOutsideLoop { span: SourceSpan },

    #[error("{span}: match is not exhaustive: a wildcard `_` arm is required")]
    NonExhaustiveMatch { span: SourceSpan },

    #[error("{span}: pattern of type `{found}` cannot match a value of type `{expected}`")]
    PatternMismatch { span: SourceSpan, expected: String, found: String },

    #[error("{span}: `{name}` expects {expected} generic argument(s), found {found}")]
    GenericArgCount { span: SourceSpan, name: String, expected: usize, found: usize },

    #[error("{span}: `{name}` is not generic")]
    NotGeneric { span: SourceSpan, name: String },

    #[error("{span}: generic type `{name}` requires explicit type arguments")]
    RequiresGenericArgs { span: SourceSpan, name: String },

    #[error("{span}: `impl` block for `{name}` must declare the same generic parameters (`impl {name}[...]`)")]
    ImplGenericMismatch { span: SourceSpan, name: String },

    #[error("{span}: `Self` may only be used inside an `impl` block")]
    SelfOutsideImpl { span: SourceSpan },

    #[error("{span}: `self` may only be used inside an `impl` method")]
    SelfOutsideMethod { span: SourceSpan },

    #[error("{span}: `{name}` is a type, not a value")]
    TypeAsValue { span: SourceSpan, name: String },

    #[error("{span}: `{name}` is a function, not a value")]
    FunctionAsValue { span: SourceSpan, name: String },

    #[error("{span}: value of type `{ty}` is not indexable")]
    NotIndexable { span: SourceSpan, ty: String },

    #[error("{span}: index must be an integer, found `{found}`")]
    IndexNotInteger { span: SourceSpan, found: String },

    #[error("{span}: cannot cast from `{from}` to `{to}`")]
    UnsupportedCast { span: SourceSpan, from: String, to: String },

    #[error("{span}: operator `{op}` requires numeric operands, found `{found}`")]
    NonNumericOperand { span: SourceSpan, op: &'static str, found: String },

    #[error("{span}: operator `{op}` requires integer operands, found `{found}`")]
    NonIntegerOperand { span: SourceSpan, op: &'static str, found: String },

    #[error("{span}: operator `{op}` requires `bool` operands, found `{found}`")]
    NonBoolOperand { span: SourceSpan, op: &'static str, found: String },

    #[error("{span}: cannot dereference a non-pointer value of type `{found}`")]
    DerefNonPointer { span: SourceSpan, found: String },

    #[error("{span}: cannot take the address of this expression")]
    InvalidAddrOf { span: SourceSpan },

    #[error("{span}: value of type `{ty}` is not iterable")]
    NotIterable { span: SourceSpan, ty: String },

    #[error("{span}: `let` requires a type annotation or an initializer")]
    MissingLetType { span: SourceSpan, name: String },
}

impl TypeError {
    pub fn span(&self) -> &SourceSpan {
        match self {
            TypeError::UndefinedVariable { span, .. }
            | TypeError::UndefinedType { span, .. }
            | TypeError::UndefinedFunction { span, .. }
            | TypeError::UndefinedMethod { span, .. }
            | TypeError::UnknownField { span, .. }
            | TypeError::UnknownFieldAccess { span, .. }
            | TypeError::NotAStruct { span, .. }
            | TypeError::PrivateMemberAccess { span, .. }
            | TypeError::InvalidOverride { span, .. }
            | TypeError::FinalOverride { span, .. }
            | TypeError::MissingField { span, .. }
            | TypeError::DuplicateFieldInLiteral { span, .. }
            | TypeError::DuplicateParam { span, .. }
            | TypeError::DuplicateStructField { span, .. }
            | TypeError::DuplicateGenericParam { span, .. }
            | TypeError::DuplicateName { span, .. }
            | TypeError::TypeMismatch { span, .. }
            | TypeError::AlreadyDefined { span, .. }
            | TypeError::AssignToImmutable { span, .. }
            | TypeError::InvalidAssignTarget { span, .. }
            | TypeError::AssignThroughConstPointer { span, .. }
            | TypeError::IfConditionNotBool { span, .. }
            | TypeError::WhileConditionNotBool { span, .. }
            | TypeError::ArgCountMismatch { span, .. }
            | TypeError::NotCallable { span, .. }
            | TypeError::MethodRequiresReceiver { span, .. }
            | TypeError::ReturnMismatch { span, .. }
            | TypeError::MissingReturnValue { span, .. }
            | TypeError::VoidReturnValue { span, .. }
            | TypeError::BreakOutsideLoopOrSwitch { span, .. }
            | TypeError::ContinueOutsideLoop { span, .. }
            | TypeError::NonExhaustiveMatch { span, .. }
            | TypeError::PatternMismatch { span, .. }
            | TypeError::GenericArgCount { span, .. }
            | TypeError::NotGeneric { span, .. }
            | TypeError::RequiresGenericArgs { span, .. }
            | TypeError::ImplGenericMismatch { span, .. }
            | TypeError::SelfOutsideImpl { span, .. }
            | TypeError::SelfOutsideMethod { span, .. }
            | TypeError::TypeAsValue { span, .. }
            | TypeError::FunctionAsValue { span, .. }
            | TypeError::NotIndexable { span, .. }
            | TypeError::IndexNotInteger { span, .. }
            | TypeError::UnsupportedCast { span, .. }
            | TypeError::NonNumericOperand { span, .. }
            | TypeError::NonIntegerOperand { span, .. }
            | TypeError::NonBoolOperand { span, .. }
            | TypeError::DerefNonPointer { span, .. }
            | TypeError::InvalidAddrOf { span, .. }
            | TypeError::NotIterable { span, .. }
            | TypeError::MissingLetType { span, .. } => span,
        }
    }
}



pub fn check_program(program: &Program) -> Vec<TypeError> {
    let mut checker = TypeChecker::new();
    checker.check(program);
    checker.errors
}


pub struct TypeChecker {
    scopes: ScopeManager,
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, Vec<String>>,
    type_aliases: HashMap<String, CType>,
    consts: HashMap<String, CType>,
    functions: HashMap<String, Vec<FuncSig>>,
    methods: HashMap<String, HashMap<String, Vec<FuncSig>>>,
    field_visibilities: HashMap<String, HashMap<String, crate::ast::Visibility>>,
    method_visibilities: HashMap<String, HashMap<String, crate::ast::Visibility>>,
    namespace_functions: HashMap<String, Vec<FuncSig>>,
    imports: Vec<Vec<String>>,
    
    current_struct: Option<String>,
    
    fn_generics: Vec<String>,
    
    return_ty: Option<CType>,
    loop_depth: usize,
    switch_depth: usize,
    errors: Vec<TypeError>,
}

impl TypeChecker {
    fn new() -> TypeChecker {
        TypeChecker {
            scopes: ScopeManager::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
            type_aliases: HashMap::new(),
            consts: HashMap::new(),
            functions: HashMap::new(),
            methods: HashMap::new(),
            field_visibilities: HashMap::new(),
            method_visibilities: HashMap::new(),
            namespace_functions: HashMap::new(),
            imports: Vec::new(),
            current_struct: None,
            fn_generics: Vec::new(),
            return_ty: None,
            loop_depth: 0,
            switch_depth: 0,
            errors: Vec::new(),
        }
    }

    
    
    

    fn check(&mut self, program: &Program) {
        
        for item in &program.items {
            match item {
                Item::Import(d) => self.imports.push(d.path.clone()),
                Item::Struct(s) => self.collect_struct(s),
                Item::Enum(e) => self.collect_enum(e),
                Item::TypeAlias(a) => self.collect_type_alias(a),
                Item::Const(c) => self.collect_const(c),
                Item::Namespace(ns) => self.collect_namespace(ns),
                _ => {}
            }
        }
        
        for item in &program.items {
            match item {
                Item::Function(f) => self.collect_function(f, None),
                Item::Impl(b) => self.collect_impl(b),
                Item::ExternFunction(e) => self.collect_extern_function(e),
                _ => {}
            }
        }
        
        for item in &program.items {
            match item {
                Item::Function(f) => self.check_function_body(f, None),
                Item::Impl(b) => {
                    for method in &b.methods {
                        self.check_function_body(method, Some(b.type_name.clone()));
                    }
                }
                _ => {}
            }
        }
    }

    
    
    

    fn collect_struct(&mut self, s: &StructDecl) {
        for dup in duplicates(&s.generics) {
            self.errors.push(TypeError::DuplicateGenericParam {
                span: s.span.clone(),
                name: dup,
            });
        }
        let generics = dedup(&s.generics);
        let mut fields = Vec::with_capacity(s.fields.len());
        let mut seen = HashMap::new();
        let mut field_visibilities = HashMap::new();
        for field in &s.fields {
            if seen.insert(field.name.clone(), ()).is_some() {
                self.errors.push(TypeError::DuplicateStructField {
                    span: field.span.clone(),
                    name: field.name.clone(),
                    ty: s.name.clone(),
                });
                continue;
            }
            self.fn_generics = generics.clone();
            fields.push((field.name.clone(), self.convert_type(&field.ty)));
            field_visibilities.insert(field.name.clone(), field.visibility);
        }
        self.fn_generics = Vec::new();
        if self.structs.contains_key(&s.name) {
            self.errors.push(TypeError::DuplicateName {
                span: s.span.clone(),
                name: s.name.clone(),
            });
            return;
        }
        let base = match &s.base {
            Some(Type::Named { name, args, .. }) => {
                if !args.is_empty() {
                    self.errors.push(TypeError::GenericArgCount {
                        span: s.span.clone(),
                        name: name.clone(),
                        expected: 0,
                        found: args.len(),
                    });
                    None
                } else if self.structs.contains_key(name) {
                    Some(name.clone())
                } else {
                    self.errors.push(TypeError::UndefinedType {
                        span: s.span.clone(),
                        name: name.clone(),
                    });
                    None
                }
            }
            Some(_) => {
                self.errors.push(TypeError::UndefinedType {
                    span: s.span.clone(),
                    name: s.name.clone(),
                });
                None
            }
            None => None,
        };
        if let Some(base_name) = &base {
            if let Some(base_info) = self.structs.get(base_name) {
                for (field_name, field_ty) in &base_info.fields {
                    if seen.insert(field_name.clone(), ()).is_some() {
                        self.errors.push(TypeError::DuplicateStructField {
                            span: s.span.clone(),
                            name: field_name.clone(),
                            ty: s.name.clone(),
                        });
                        continue;
                    }
                    fields.push((field_name.clone(), field_ty.clone()));
                }
            }
        }
        self.structs.insert(
            s.name.clone(),
            StructInfo {
                name: s.name.clone(),
                generics,
                base,
                is_final: s.is_final,
                is_abstract: s.is_abstract,
                fields,
            },
        );
        self.field_visibilities.insert(s.name.clone(), field_visibilities);

        for method in &s.methods {
            self.collect_function(method, Some(s.name.clone()));
        }
    }

    fn collect_enum(&mut self, e: &EnumDecl) {
        let variant_names: Vec<String> = e.variants.iter().map(|v| v.name.clone()).collect();
        for dup in duplicates(&variant_names) {
            self.errors.push(TypeError::DuplicateName {
                span: e.span.clone(),
                name: dup,
            });
        }
        let variants = dedup(&variant_names);
        if self.structs.contains_key(&e.name) {
            self.errors.push(TypeError::DuplicateName {
                span: e.span.clone(),
                name: e.name.clone(),
            });
            return;
        }
        self.structs.insert(
            e.name.clone(),
            StructInfo {
                name: e.name.clone(),
                generics: Vec::new(),
                base: None,
                is_final: false,
                is_abstract: false,
                fields: Vec::new(),
            },
        );
        self.enums.insert(e.name.clone(), variants);
    }

    fn collect_type_alias(&mut self, a: &TypeAliasDecl) {
        if self.type_aliases.contains_key(&a.name) {
            self.errors.push(TypeError::DuplicateName {
                span: a.span.clone(),
                name: a.name.clone(),
            });
            return;
        }
        let target = self.convert_type(&a.target);
        self.type_aliases.insert(a.name.clone(), target);
    }

    fn collect_const(&mut self, c: &ConstDecl) {
        if self.consts.contains_key(&c.name) {
            self.errors.push(TypeError::DuplicateName {
                span: c.span.clone(),
                name: c.name.clone(),
            });
            return;
        }
        let declared = self.convert_type(&c.ty);
        let inferred = self.check_expr(&c.value, Some(&declared));
        match CType::unify(&inferred, &declared) {
            Some(u) => {
                self.consts.insert(c.name.clone(), u);
            }
            None => {
                self.errors.push(TypeError::TypeMismatch {
                    span: c.value.span().clone(),
                    expected: declared.to_string(),
                    found: inferred.to_string(),
                });
                self.consts.insert(c.name.clone(), declared);
            }
        }
    }

    fn collect_impl(&mut self, b: &ImplBlock) {
        if !self.structs.contains_key(&b.type_name) {
            self.errors.push(TypeError::UndefinedType {
                span: b.span.clone(),
                name: b.type_name.clone(),
            });
            return;
        }
        let struct_generics = self
            .structs
            .get(&b.type_name)
            .map(|info| info.generics.clone())
            .unwrap_or_default();
        if b.generics != struct_generics {
            self.errors.push(TypeError::ImplGenericMismatch {
                span: b.span.clone(),
                name: b.type_name.clone(),
            });
            return;
        }
        for method in &b.methods {
            self.collect_function(method, Some(b.type_name.clone()));
        }
    }

    fn collect_extern_function(&mut self, e: &ExternFunctionDecl) {
        self.fn_generics = Vec::new();
        let has_self = matches!(e.params.first(), Some(p) if p.name == "self");
        let sig = FuncSig {
            name: e.name.clone(),
            generics: Vec::new(),
            params: e.params.iter().map(|p| self.convert_type(&p.ty)).collect(),
            has_self,
            is_const: false,
            is_virtual: false,
            is_override: false,
            is_final: false,
            is_abstract: false,
            is_static: true,
            return_ty: e.return_ty.as_ref().map(|t| self.convert_type(t)),
            struct_name: None,
            is_builtin: false,
            is_extern: false,
            is_variadic: e.is_variadic,
        };
        let overloads = self.functions.entry(e.name.clone()).or_default();
        
        
        
        if !overloads.contains(&sig) {
            overloads.push(sig);
        }
    }

    fn collect_namespace(&mut self, ns: &NamespaceDecl) {
        for item in &ns.items {
            match item {
                Item::Function(f) => {
                    let qualified = format!("{}::{}", ns.name, f.name);
                    let has_self = matches!(f.params.first(), Some(p) if p.name == "self");
                    let sig = FuncSig {
                        name: f.name.clone(),
                        generics: dedup(&f.generics),
                        params: f.params.iter().map(|p| self.convert_type(&p.ty)).collect(),
                        has_self,
                        is_const: f.is_const,
                        is_virtual: f.is_virtual || f.is_abstract,
                        is_override: f.is_override,
                        is_final: f.is_final,
                        is_abstract: f.is_abstract,
                        is_static: f.is_static,
                        return_ty: f.return_ty.as_ref().map(|t| self.convert_type(t)),
                        struct_name: None,
                        is_builtin: false,
            is_extern: false,
            is_variadic: false,
                    };
                    self.namespace_functions.entry(qualified).or_default().push(sig);
                }
                Item::Struct(s) => self.collect_struct(s),
                Item::Namespace(inner) => self.collect_namespace(inner),
                _ => {}
            }
        }
    }

    fn collect_function(&mut self, f: &Function, struct_name: Option<String>) {
        for dup in duplicates(&f.generics) {
            self.errors.push(TypeError::DuplicateGenericParam {
                span: f.span.clone(),
                name: dup,
            });
        }
        for dup in duplicates(&f.params.iter().map(|p| p.name.clone()).collect::<Vec<_>>()) {
            self.errors.push(TypeError::DuplicateParam {
                span: f.span.clone(),
                name: dup,
            });
        }
        
        
        let struct_generics = match &struct_name {
            Some(name) => self.structs.get(name).map(|info| info.generics.clone()).unwrap_or_default(),
            None => Vec::new(),
        };
        self.fn_generics = dedup(&struct_generics)
            .into_iter()
            .chain(f.generics.clone())
            .collect();
        self.current_struct = struct_name.clone();
        let params = f.params.iter().map(|p| self.convert_type(&p.ty)).collect();
        let return_ty = f.return_ty.as_ref().map(|t| self.convert_type(t));
        self.fn_generics = Vec::new();
        self.current_struct = None;

        let has_self = matches!(f.params.first(), Some(p) if p.name == "self");
        let sig = FuncSig {
            name: f.name.clone(),
            generics: dedup(&f.generics),
            params,
            has_self,
            is_const: f.is_const,
            is_virtual: f.is_virtual || f.is_abstract,
            is_override: f.is_override,
            is_final: f.is_final,
            is_abstract: f.is_abstract,
            is_static: f.is_static,
            return_ty,
            struct_name: struct_name.clone(),
            is_builtin: false,
            is_extern: false,
            is_variadic: false,
        };
        if let Some(ty) = struct_name.clone() {
            let methods = self.methods.entry(ty.clone()).or_default();
            methods.entry(f.name.clone()).or_default().push(sig.clone());
            self.method_visibilities.entry(ty.clone()).or_default().insert(f.name.clone(), f.visibility);
            if f.is_override {
                self.check_override_rule(&ty, f);
            }
        } else {
            self.functions.entry(f.name.clone()).or_default().push(sig);
        }
    }

    fn check_override_rule(&mut self, struct_name: &str, method: &Function) {
        let mut current = Some(struct_name.to_string());
        let mut base_name = None;
        while let Some(name) = current {
            if let Some(info) = self.structs.get(&name) {
                if let Some(parent) = &info.base {
                    base_name = Some(parent.clone());
                    let parent_methods = self.methods.get(parent).and_then(|m| m.get(&method.name));
                    if let Some(overloads) = parent_methods {
                        if overloads.iter().any(|sig| sig.is_virtual || sig.is_abstract) {
                            if overloads.iter().any(|sig| sig.is_final) {
                                self.errors.push(TypeError::FinalOverride {
                                    span: method.span.clone(),
                                    name: format!("{}::{}", parent, method.name),
                                });
                            }
                            return;
                        }
                        if overloads.iter().any(|sig| sig.is_final) {
                            self.errors.push(TypeError::FinalOverride {
                                span: method.span.clone(),
                                name: format!("{}::{}", parent, method.name),
                            });
                        }
                        break;
                    }
                }
            }
            current = self.structs.get(&name).and_then(|info| info.base.clone());
            if current.is_none() {
                break;
            }
        }

        if let Some(base) = &base_name {
            let base_info = self.structs.get(base);
            if base_info.is_some() && base_info.is_some_and(|info| info.is_final) {
                self.errors.push(TypeError::FinalOverride {
                    span: method.span.clone(),
                    name: base.clone(),
                });
                return;
            }
        }

        let base_display = base_name.unwrap_or_else(|| struct_name.to_string());
        self.errors.push(TypeError::InvalidOverride {
            span: method.span.clone(),
            method: method.name.clone(),
            base: base_display,
        });
    }

    
    
    

    
    
    fn convert_type(&mut self, ty: &Type) -> CType {
        match ty {
            Type::Primitive(p) => CType::from_primitive(*p),
            Type::Pointer { pointee, mutability, .. } => {
                let inner = self.convert_type(pointee);
                CType::Pointer {
                    pointee: Box::new(inner),
                    mutable: *mutability == PointerMutability::Mut,
                }
            }
            Type::Named { name, args, span } => {
                if name == "Self" {
                    return match &self.current_struct {
                        Some(struct_name) => {
                            let generics = self.structs.get(struct_name).map(|i| i.generics.clone()).unwrap_or_default();
                            CType::Struct {
                                name: struct_name.clone(),
                                args: generics.into_iter().map(CType::Generic).collect(),
                            }
                        }
                        None => {
                            self.errors.push(TypeError::SelfOutsideImpl {
                                span: span.clone(),
                            });
                            CType::Void
                        }
                    };
                }
                if self.type_aliases.contains_key(name) {
                    let alias = self.type_aliases.get(name).cloned().unwrap();
                    if args.is_empty() {
                        return alias;
                    }
                    self.errors.push(TypeError::GenericArgCount {
                        span: span.clone(),
                        name: name.clone(),
                        expected: 0,
                        found: args.len(),
                    });
                    return CType::Void;
                }
                if self.fn_generics.iter().any(|g| g == name) {
                    return CType::Generic(name.clone());
                }
                let Some(info) = self.structs.get(name).cloned() else {
                    self.errors.push(TypeError::UndefinedType {
                        span: span.clone(),
                        name: name.clone(),
                    });
                    return CType::Void;
                };
                if args.is_empty() {
                    if info.generics.is_empty() {
                        CType::Struct { name: name.clone(), args: Vec::new() }
                    } else {
                        self.errors.push(TypeError::RequiresGenericArgs {
                            span: span.clone(),
                            name: name.clone(),
                        });
                        CType::Void
                    }
                } else {
                    if info.generics.is_empty() {
                        self.errors.push(TypeError::NotGeneric {
                            span: span.clone(),
                            name: name.clone(),
                        });
                        return CType::Void;
                    }
                    if args.len() != info.generics.len() {
                        self.errors.push(TypeError::GenericArgCount {
                            span: span.clone(),
                            name: name.clone(),
                            expected: info.generics.len(),
                            found: args.len(),
                        });
                        return CType::Void;
                    }
                    CType::Struct {
                        name: name.clone(),
                        args: args.iter().map(|a| self.convert_type(a)).collect(),
                    }
                }
            }
            Type::Array { element, length, span } => {
                let elem_ty = self.convert_type(element);
                match self.check_array_length(length, span) {
                    Ok(len) => CType::Array {
                        element: Box::new(elem_ty),
                        length: len,
                    },
                    Err(ty) => ty,
                }
            }
        }
    }

    
    fn check_array_length(&mut self, expr: &Expr, span: &SourceSpan) -> Result<u64, CType> {
        match expr {
            Expr::Literal(LiteralExpr::Integer(n), _) if *n >= 0 => Ok(*n as u64),
            _ => {
                self.errors.push(TypeError::TypeMismatch {
                    expected: "compile-time integer constant for array length".to_string(),
                    found: format!("{expr:?}"),
                    span: span.clone(),
                });
                Err(CType::Void)
            }
        }
    }

    
    
    

    fn check_function_body(&mut self, f: &Function, struct_name: Option<String>) {
        self.current_struct = struct_name.clone();
        let struct_generics = match &struct_name {
            Some(name) => self.structs.get(name).map(|info| info.generics.clone()).unwrap_or_default(),
            None => Vec::new(),
        };
        self.fn_generics = struct_generics.into_iter().chain(f.generics.clone()).collect();
        self.return_ty = f.return_ty.as_ref().map(|t| self.convert_type(t));
        self.scopes.push_scope();
        for param in &f.params {
            
            let ty = self.convert_type(&param.ty);
            self.scopes.define(
                param.name.clone(),
                VarInfo {
                    ty,
                    mutable: false,
                },
            );
        }
        self.check_block(&f.body);
        self.scopes.pop_scope();
        self.current_struct = None;
        self.fn_generics = Vec::new();
        self.return_ty = None;
    }

    fn check_block(&mut self, block: &Block) {
        self.scopes.push_scope();
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
        self.scopes.pop_scope();
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(s) => self.check_let(s),
            Stmt::Expr(e) => {
                self.check_expr(e, None);
            }
            Stmt::If(s) => self.check_if(s),
            Stmt::While(s) => {
                let cond = self.check_expr(&s.condition, None);
                if !matches!(cond, CType::Bool) {
                    self.errors.push(TypeError::WhileConditionNotBool {
                        span: s.condition.span().clone(),
                        found: cond.to_string(),
                    });
                }
                self.loop_depth += 1;
                self.check_block(&s.body);
                self.loop_depth -= 1;
            }
            Stmt::For(s) => {
                let iterable = self.check_expr(&s.iterable, None);
                let elem = match &iterable {
                    CType::Pointer { pointee, .. } => (**pointee).clone(),
                    CType::Str => CType::Char,
                    CType::Struct { name, args } if name == "Vec" => {
                        match args.first() {
                            Some(t) => t.clone(),
                            None => {
                                self.errors.push(TypeError::NotIterable {
                                    span: s.iterable.span().clone(),
                                    ty: iterable.to_string(),
                                });
                                CType::Void
                            }
                        }
                    }
                    CType::Struct { name, .. } if name == "String" => CType::Char,
                    other => {
                        self.errors.push(TypeError::NotIterable {
                            span: s.iterable.span().clone(),
                            ty: other.to_string(),
                        });
                        CType::Void
                    }
                };
                self.loop_depth += 1;
                self.scopes.push_scope();
                self.scopes.define(
                    s.variable.clone(),
                    VarInfo {
                        ty: elem,
                        mutable: false,
                    },
                );
                self.check_block(&s.body);
                self.scopes.pop_scope();
                self.loop_depth -= 1;
            }
            Stmt::Switch(s) => {
                let value = self.check_expr(&s.value, None);
                let _ = value;
                self.switch_depth += 1;
                for arm in &s.arms {
                    for stmt in &arm.body {
                        self.check_stmt(stmt);
                    }
                }
                self.switch_depth -= 1;
            }
            Stmt::Break(s) => {
                if self.loop_depth == 0 && self.switch_depth == 0 {
                    self.errors.push(TypeError::BreakOutsideLoopOrSwitch { span: s.span.clone() });
                }
            }
            Stmt::Continue(s) => {
                if self.loop_depth == 0 {
                    self.errors.push(TypeError::ContinueOutsideLoop { span: s.span.clone() });
                }
            }
            Stmt::Block(b) => self.check_block(b),
        }
    }

    fn check_let(&mut self, s: &LetStmt) {
        let declared = s.ty.as_ref().map(|t| self.convert_type(t));
        let mut ty = match (&declared, &s.init) {
            (None, None) => {
                self.errors.push(TypeError::MissingLetType {
                    span: s.span.clone(),
                    name: s.name.clone(),
                });
                CType::Void
            }
            (Some(t), _) => t.clone(),
            (None, Some(init)) => {
                let inferred = self.check_expr(init, None);
                self.scopes.define(
                    s.name.clone(),
                    VarInfo {
                        ty: inferred.clone(),
                        mutable: s.mutable,
                    },
                );
                return;
            }
        };
        if let Some(init) = &s.init {
            let init_ty = self.check_expr(init, Some(&ty));
            match CType::unify(&init_ty, &ty) {
                Some(u) => ty = u,
                None => {
                    self.errors.push(TypeError::TypeMismatch {
                        span: init.span().clone(),
                        expected: ty.to_string(),
                        found: init_ty.to_string(),
                    });
                    ty = declared.unwrap_or(CType::Void);
                }
            }
        }
        if !self.scopes.define(
            s.name.clone(),
            VarInfo {
                ty,
                mutable: s.mutable,
            },
        ) {
            self.errors.push(TypeError::AlreadyDefined {
                span: s.span.clone(),
                name: s.name.clone(),
            });
        }
    }

    fn check_if(&mut self, s: &IfStmt) {
        let cond = self.check_expr(&s.condition, None);
        if !matches!(cond, CType::Bool) {
            self.errors.push(TypeError::IfConditionNotBool {
                span: s.condition.span().clone(),
                found: cond.to_string(),
            });
        }
        self.check_block(&s.then_block);
        match &s.else_branch {
            Some(ElseBranch::If(inner)) => self.check_if(inner),
            Some(ElseBranch::Block(b)) => self.check_block(b),
            None => {}
        }
    }

    
    
    

    
    
    
    fn check_expr(&mut self, expr: &Expr, expected: Option<&CType>) -> CType {
        let result = match expr {
            Expr::Literal(lit, span) => {
                let ty = literal_type(lit, expected);
                if let Some(exp) = expected {
                    match CType::unify(&ty, exp) {
                        Some(u) => u,
                        None => {
                            self.errors.push(TypeError::TypeMismatch {
                                span: span.clone(),
                                expected: exp.to_string(),
                                found: ty.to_string(),
                            });
                            exp.clone()
                        }
                    }
                } else {
                    ty
                }
            }
            Expr::Var { name, span } => self.check_var(name, span),
            Expr::Path { segments, span } => {
                if segments.len() == 2 {
                    let enum_name = &segments[0];
                    let variant_name = &segments[1];
                    if let Some(variants) = self.enums.get(enum_name)
                        && variants.iter().any(|v| v == variant_name)
                    {
                        return CType::I32;
                    }
                }
                let name = segments.join("::");
                self.errors.push(TypeError::FunctionAsValue {
                    span: span.clone(),
                    name,
                });
                CType::Void
            }
            Expr::Call { callee, args, span } => self.check_call(callee, args, span),
            Expr::MethodCall { receiver, method, args, span } => {
                self.check_method_call(receiver, method, args, span)
            }
            Expr::FieldAccess { base, field, span } => self.check_field_access(base, field, span),
            Expr::Index { base, index, span } => self.check_index(base, index, span),
            Expr::Unary { op, operand, span } => self.check_unary(*op, operand, span),
            Expr::Binary { op, lhs, rhs, span } => self.check_binary(*op, lhs, rhs, span),
            Expr::Assign { target, op, value, span } => self.check_assign(target, *op, value, span),
            Expr::Cast { expr: inner, ty, span } => self.check_cast(inner, &*ty, span),
            Expr::StructLiteral { name, args, fields, span } => {
                self.check_struct_literal(name, args, fields, span, expected)
            }
            Expr::Match { scrutinee, arms, span } => self.check_match(scrutinee, arms, span, expected),
            Expr::Return { value, span } => self.check_return(value.as_deref(), span),
            Expr::Range { start, end, inclusive, span } => {
                self.check_range(start, end, *inclusive, span)
            }
            Expr::ArrayRepeat { value, count, span } => {
                self.check_array_repeat(value, count, span, expected)
            }
            Expr::ArrayLiteral { elements, span } => {
                self.check_array_literal(elements, span, expected)
            }
            Expr::SizeOf { ty, .. } => {
                
                
                
                self.convert_type(ty);
                CType::Usize
            }
        };
        
        match expr {
            Expr::Literal(..)
            | Expr::Call { .. }
            | Expr::MethodCall { .. }
            | Expr::FieldAccess { .. }
            | Expr::Index { .. }
            | Expr::Unary { .. }
            | Expr::Binary { .. }
            | Expr::Assign { .. }
            | Expr::Cast { .. }
            | Expr::StructLiteral { .. }
            | Expr::Match { .. }
            | Expr::ArrayRepeat { .. }
            | Expr::ArrayLiteral { .. }
            | Expr::SizeOf { .. }
            | Expr::Range { .. } => {
                if let Some(exp) = expected
                    && CType::unify(&result, exp).is_none()
                {
                    self.errors.push(TypeError::TypeMismatch {
                        span: expr.span().clone(),
                        expected: exp.to_string(),
                        found: result.to_string(),
                    });
                }
            }
            _ => {}
        }
        result
    }

    fn check_var(&mut self, name: &str, span: &SourceSpan) -> CType {
        match name {
            "Self" => {
                self.errors.push(TypeError::TypeAsValue {
                    span: span.clone(),
                    name: "Self".to_string(),
                });
                CType::Void
            }
            "self" => match self.scopes.lookup("self") {
                Some(info) => info.ty.clone(),
                None => {
                    self.errors.push(TypeError::SelfOutsideMethod {
                        span: span.clone(),
                    });
                    CType::Void
                }
            },
            _ => {
                if self.structs.contains_key(name) {
                    self.errors.push(TypeError::TypeAsValue {
                        span: span.clone(),
                        name: name.to_string(),
                    });
                    return CType::Void;
                }
                match self.scopes.lookup(name) {
                    Some(info) => info.ty.clone(),
                    None => match self.consts.get(name) {
                        Some(ty) => ty.clone(),
                        None => {
                            self.errors.push(TypeError::UndefinedVariable {
                                span: span.clone(),
                                name: name.to_string(),
                            });
                            CType::Void
                        }
                    },
                }
            }
        }
    }

    fn check_call(&mut self, callee: &Expr, args: &[Expr], span: &SourceSpan) -> CType {
        let opt_sig = match callee {
            Expr::Var { name, .. } => {
                if name == "println" || name == "print" {
                    if args.is_empty() {
                        self.errors.push(TypeError::ArgCountMismatch {
                            span: span.clone(),
                            expected: 1,
                            found: 0,
                        });
                    } else if let Expr::Literal(LiteralExpr::String(s), _) = &args[0] {
                        let placeholders = s.matches("{}").count();
                        let extra = args.len() - 1;
                        if placeholders > 0 && placeholders != extra {
                            self.errors.push(TypeError::ArgCountMismatch {
                                span: span.clone(),
                                expected: placeholders,
                                found: extra,
                            });
                        }
                    }
                    return CType::Void;
                }
                if let Some(overloads) = self.functions.get(name).cloned() {
                    return match self.resolve_function_overload(&overloads, args, span, name) {
                        Some(sig) => self.check_call_against(sig, args, span),
                        None => CType::Void,
                    };
                }
                if self.structs.contains_key(name) {
                    let overloads = self
                        .methods
                        .get(name)
                        .and_then(|m| m.get("new"))
                        .cloned()
                        .unwrap_or_default();
                    if !overloads.is_empty() {
                        return match self.resolve_function_overload(&overloads, args, span, name) {
                            Some(sig) => self.check_call_against(sig, args, span),
                            None => CType::Void,
                        };
                    }
                    self.errors.push(TypeError::TypeAsValue {
                        span: span.clone(),
                        name: name.clone(),
                    });
                    return CType::Void;
                }
                self.errors.push(TypeError::UndefinedFunction {
                    span: span.clone(),
                    name: name.clone(),
                });
                return CType::Void;
            }
            Expr::Path { segments, .. } => self.resolve_path_call(segments, span),
            other => {
                self.errors.push(TypeError::NotCallable {
                    span: span.clone(),
                    name: other.span().to_string(),
                });
                return CType::Void;
            }
        };

        match opt_sig {
            Some(sig) => self.check_call_against(sig, args, span),
            None => CType::Void,
        }
    }

    fn resolve_function_overload(
        &mut self,
        overloads: &[FuncSig],
        args: &[Expr],
        span: &SourceSpan,
        name: &str,
    ) -> Option<FuncSig> {
        let mut best: Option<FuncSig> = None;
        let mut matches = 0usize;
        let mut arity_candidates = Vec::new();
        for sig in overloads {
            let ok_arity = if sig.is_variadic {
                args.len() >= sig.params.len()
            } else {
                sig.params.len() == args.len()
            };
            if ok_arity {
                arity_candidates.push(sig);
            }
        }

        for sig in &arity_candidates {
            let before = self.errors.len();
            let mut compatible = true;
            for (param, arg) in sig.params.iter().zip(args.iter()) {
                let arg_ty = self.check_expr(arg, Some(param));
                if CType::unify_arg(param, &arg_ty).is_none() {
                    compatible = false;
                    break;
                }
            }
            self.errors.truncate(before);
            if compatible {
                matches += 1;
                best = Some((*sig).clone());
            }
        }

        if matches == 0 {
            if arity_candidates.is_empty() {
                self.errors.push(TypeError::ArgCountMismatch {
                    span: span.clone(),
                    expected: overloads.first().map(|s| s.params.len()).unwrap_or(0),
                    found: args.len(),
                });
                return None;
            }
            let sig = &arity_candidates[0];
            for (param, arg) in sig.params.iter().zip(args.iter()) {
                self.check_expr(arg, Some(param));
            }
            return None;
        }
        if matches > 1 {
            self.errors.push(TypeError::UndefinedFunction {
                span: span.clone(),
                name: name.to_string(),
            });
            return None;
        }
        best
    }

    fn resolve_path_call(&mut self, segments: &[String], span: &SourceSpan) -> Option<FuncSig> {
        if segments.first().map(String::as_str) == Some("Self") {
            let struct_name = match &self.current_struct {
                Some(s) => s.clone(),
                None => {
                    self.errors.push(TypeError::SelfOutsideImpl {
                        span: span.clone(),
                    });
                    return None;
                }
            };
            if let Some(overloads) = self.methods.get(&struct_name).and_then(|m| {
                segments.get(1).and_then(|name| m.get(name))
            }) {
                let mut best = None;
                for sig in overloads {
                    if sig.has_self {
                        self.errors.push(TypeError::MethodRequiresReceiver {
                            span: span.clone(),
                            name: sig.name.clone(),
                        });
                    }
                    best = Some(sig.clone());
                }
                return best;
            }
            self.errors.push(TypeError::UndefinedMethod {
                span: span.clone(),
                name: segments.last().cloned().unwrap_or_default(),
                ty: struct_name,
            });
            return None;
        }
        let qualified = segments.join("::");
        if let Some(overloads) = self.namespace_functions.get(&qualified) {
            return overloads.first().cloned();
        }
        if segments.len() == 2 && self.structs.contains_key(&segments[0]) {
            let ty = segments[0].clone();
            let name = segments[1].clone();
            if let Some(overloads) = self.methods.get(&ty).and_then(|m| m.get(&name)) {
                let mut best = None;
                for sig in overloads {
                    if sig.has_self {
                        self.errors.push(TypeError::MethodRequiresReceiver {
                            span: span.clone(),
                            name: sig.name.clone(),
                        });
                    }
                    best = Some(sig.clone());
                }
                return best;
            }
            self.errors.push(TypeError::UndefinedMethod {
                span: span.clone(),
                name,
                ty,
            });
            return None;
        }
        
        let last = segments.last().cloned().unwrap_or_default();
        let prefix = &segments[..segments.len().saturating_sub(1)];
        let imported = self.imports.iter().any(|p| p.starts_with(prefix));
        if imported && (last == "println" || last == "print") {
            return Some(FuncSig {
                name: last,
                generics: Vec::new(),
                params: Vec::new(),
                has_self: false,
                is_const: false,
                is_virtual: false,
                is_override: false,
                is_final: false,
                is_abstract: false,
                is_static: false,
                return_ty: None,
                struct_name: None,
                is_builtin: true,
            is_extern: false,
            is_variadic: false,
            });
        }
        self.errors.push(TypeError::UndefinedFunction {
            span: span.clone(),
            name: segments.join("::"),
        });
        None
    }

    fn check_call_against(&mut self, sig: FuncSig, args: &[Expr], span: &SourceSpan) -> CType {
        let ok_count = if sig.is_variadic {
            args.len() >= sig.params.len()
        } else {
            args.len() == sig.params.len()
        };
        if !sig.is_builtin && !ok_count {
            self.errors.push(TypeError::ArgCountMismatch {
                span: span.clone(),
                expected: sig.params.len(),
                found: args.len(),
            });
        }
        for (i, arg) in args.iter().enumerate() {
            let expected = sig.params.get(i);
            self.check_expr(arg, expected);
        }
        sig.return_ty.unwrap_or(CType::Void)
    }

    fn check_method_call(
        &mut self,
        receiver: &Expr,
        method: &str,
        args: &[Expr],
        span: &SourceSpan,
    ) -> CType {
        let receiver_ty = self.check_expr(receiver, None);
        let (struct_name, _) = match &receiver_ty {
            CType::Struct { name, .. } => (name.clone(), receiver_ty.clone()),
            CType::Pointer { pointee, .. } => match &**pointee {
                CType::Struct { name, .. } => (name.clone(), (**pointee).clone()),
                other => {
                    self.errors.push(TypeError::UndefinedMethod {
                        span: span.clone(),
                        name: method.to_string(),
                        ty: other.to_string(),
                    });
                    return CType::Void;
                }
            },
            other => {
                self.errors.push(TypeError::UndefinedMethod {
                    span: span.clone(),
                    name: method.to_string(),
                    ty: other.to_string(),
                });
                return CType::Void;
            }
        };
        let overloads = self
            .methods
            .get(&struct_name)
            .and_then(|m| m.get(method))
            .cloned()
            .unwrap_or_default();
        if overloads.is_empty() {
            self.errors.push(TypeError::UndefinedMethod {
                span: span.clone(),
                name: method.to_string(),
                ty: struct_name,
            });
            return CType::Void;
        }
        let member_visibility = self.method_visibilities.get(&struct_name).and_then(|m| m.get(method)).copied();
        if let Some(visibility) = member_visibility
            && !self.can_access_member(&struct_name, self.current_struct.as_deref(), visibility)
        {
            self.errors.push(TypeError::PrivateMemberAccess {
                span: span.clone(),
                member: method.to_string(),
                ty: struct_name.clone(),
            });
        }

        let mut best: Option<FuncSig> = None;
        let mut arity_candidates = Vec::new();
        for sig in &overloads {
            if sig.params.len() == 1 + args.len() {
                arity_candidates.push(sig);
            }
        }
        for sig in &arity_candidates {
            let Some(self_param) = sig.params.first() else { continue; };
            let before = self.errors.len();
            let receiver_ok = self.receiver_matches_method(&receiver_ty, self_param);
            let mut compatible = receiver_ok;
            if compatible {
                for (i, arg) in args.iter().enumerate() {
                    let expected = sig.params.get(i + 1);
                    let arg_ty = self.check_expr(arg, expected);
                    if expected.map(|e| CType::unify_arg(e, &arg_ty).is_none()).unwrap_or(false) {
                        compatible = false;
                        break;
                    }
                }
            }
            self.errors.truncate(before);
            if compatible {
                best = Some((*sig).clone());
                break;
            }
        }
        let Some(sig) = best else {
            if arity_candidates.is_empty() {
                self.errors.push(TypeError::ArgCountMismatch {
                    span: span.clone(),
                    expected: overloads.first().map(|s| s.params.len().saturating_sub(1)).unwrap_or(0),
                    found: args.len(),
                });
            } else {
                let sig = &arity_candidates[0];
                let receiver_ok =
                    self.receiver_matches_method(&receiver_ty, sig.params.first().unwrap_or(&CType::Void));
                if receiver_ok {
                    for (i, arg) in args.iter().enumerate() {
                        let expected = sig.params.get(i + 1);
                        self.check_expr(arg, expected);
                    }
                } else {
                    self.errors.push(TypeError::UndefinedMethod {
                        span: span.clone(),
                        name: method.to_string(),
                        ty: struct_name.clone(),
                    });
                }
            }
            return CType::Void;
        };
        for (i, arg) in args.iter().enumerate() {
            let expected = sig.params.get(i + 1);
            self.check_expr(arg, expected);
        }
        
        
        
        let bindings = self
            .bind_receiver_args(&receiver_ty, sig.params.first().unwrap_or(&CType::Void));
        sig.return_ty.clone().unwrap_or(CType::Void).substitute(&bindings)
    }

    
    
    
    fn receiver_matches_method(&self, receiver_ty: &CType, self_param: &CType) -> bool {
        if CType::unify(receiver_ty, self_param).is_some() {
            return true;
        }
        if CType::unify_arg(self_param, receiver_ty).is_some() {
            return true;
        }
        match (receiver_ty, self_param) {
            (CType::Pointer { pointee, .. }, p) => CType::unify(p, pointee).is_some(),
            (r, CType::Pointer { pointee, .. }) => CType::unify(r, pointee).is_some(),
            _ => false,
        }
    }

    
    
    fn bind_receiver_args(&self, receiver_ty: &CType, self_param: &CType) -> HashMap<String, CType> {
        fn inner(bindings: &mut HashMap<String, CType>, p: &CType, a: &CType) {
            use CType::{Generic, Pointer, Struct};
            match (p, a) {
                (Generic(name), concrete) => {
                    bindings.insert(name.clone(), concrete.clone());
                }
                (Struct { name, args }, Struct { name: an, args: aargs }) if name == an => {
                    for (pa, aa) in args.iter().zip(aargs.iter()) {
                        inner(bindings, pa, aa);
                    }
                }
                
                
                
                
                (Pointer { pointee, .. }, _) => inner(bindings, pointee, a),
                (_, Pointer { pointee, .. }) => inner(bindings, p, pointee),
                _ => {}
            }
        }
        let mut bindings = HashMap::new();
        inner(&mut bindings, self_param, receiver_ty);
        bindings
    }

    fn can_access_member(&self, owner: &str, current: Option<&str>, visibility: crate::ast::Visibility) -> bool {
        match visibility {
            crate::ast::Visibility::Public => true,
            crate::ast::Visibility::Private => {
                current
                    .map(|current_struct| current_struct == owner)
                    .unwrap_or(false)
            }
            crate::ast::Visibility::Protected => {
                let Some(current_struct) = current else {
                    return false;
                };
                if current_struct == owner {
                    return true;
                }
                let mut cursor = Some(current_struct.to_string());
                while let Some(name) = cursor {
                    if name == owner {
                        return true;
                    }
                    cursor = self.structs.get(&name).and_then(|info| info.base.clone());
                }
                false
            }
        }
    }

    fn check_field_access(&mut self, base: &Expr, field: &str, span: &SourceSpan) -> CType {
        let base_ty = self.check_expr(base, None);
        let (struct_name, struct_args, fields) = match &base_ty {
            CType::Struct { name, args } => {
                let info = self.structs.get(name).expect("struct is registered");
                (name.clone(), args.clone(), info.fields.clone())
            }
            CType::Pointer { pointee, .. } => match &**pointee {
                CType::Struct { name, args } => {
                    let info = self.structs.get(name).expect("struct is registered");
                    (name.clone(), args.clone(), info.fields.clone())
                }
                other => {
                    self.errors.push(TypeError::NotAStruct {
                        span: span.clone(),
                        ty: other.to_string(),
                    });
                    return CType::Void;
                }
            },
            other => {
                self.errors.push(TypeError::NotAStruct {
                    span: span.clone(),
                    ty: other.to_string(),
                });
                return CType::Void;
            }
        };
        let bindings: HashMap<String, CType> = self
            .structs
            .get(&struct_name)
            .map(|info| {
                info.generics
                    .iter()
                    .zip(struct_args.iter())
                    .map(|(g, ty)| (g.clone(), ty.clone()))
                    .collect()
            })
            .unwrap_or_default();
        let visibility = self.field_visibilities.get(&struct_name).and_then(|m| m.get(field)).copied();
        if let Some(visibility) = visibility
            && !self.can_access_member(&struct_name, self.current_struct.as_deref(), visibility)
        {
            self.errors.push(TypeError::PrivateMemberAccess {
                span: span.clone(),
                member: field.to_string(),
                ty: struct_name.clone(),
            });
            return CType::Void;
        }
        fields
            .into_iter()
            .find(|(name, _)| name == field)
            .map(|(_, ty)| ty.substitute(&bindings))
            .unwrap_or_else(|| {
                self.errors.push(TypeError::UnknownFieldAccess {
                    span: span.clone(),
                    ty: struct_name,
                    field: field.to_string(),
                });
                CType::Void
            })
    }

    fn check_index(&mut self, base: &Expr, index: &Expr, span: &SourceSpan) -> CType {
        let base_ty = self.check_expr(base, None);
        let index_ty = self.check_expr(index, None);
        if !index_ty.accepts_int() {
            self.errors.push(TypeError::IndexNotInteger {
                span: index.span().clone(),
                found: index_ty.to_string(),
            });
        }
        match &base_ty {
            CType::Pointer { pointee, .. } => (**pointee).clone(),
            CType::Array { element, .. } => (**element).clone(),
            other => {
                self.errors.push(TypeError::NotIndexable {
                    span: span.clone(),
                    ty: other.to_string(),
                });
                CType::Void
            }
        }
    }

    fn check_unary(&mut self, op: UnaryOp, operand: &Expr, span: &SourceSpan) -> CType {
        match op {
            UnaryOp::Neg => {
                let ty = self.check_expr(operand, None);
                if !ty.accepts_numeric() {
                    self.errors.push(TypeError::NonNumericOperand {
                        span: span.clone(),
                        op: "-",
                        found: ty.to_string(),
                    });
                }
                ty
            }
            UnaryOp::Not => {
                let ty = self.check_expr(operand, None);
                if !matches!(ty, CType::Bool) {
                    self.errors.push(TypeError::NonBoolOperand {
                        span: span.clone(),
                        op: "!",
                        found: ty.to_string(),
                    });
                }
                CType::Bool
            }
            UnaryOp::BitNot => {
                let ty = self.check_expr(operand, None);
                if !ty.accepts_int() {
                    self.errors.push(TypeError::NonIntegerOperand {
                        span: span.clone(),
                        op: "~",
                        found: ty.to_string(),
                    });
                }
                ty
            }
            UnaryOp::Deref => {
                let ty = self.check_expr(operand, None);
                match &ty {
                    CType::Pointer { pointee, .. } => (**pointee).clone(),
                    other => {
                        self.errors.push(TypeError::DerefNonPointer {
                            span: span.clone(),
                            found: other.to_string(),
                        });
                        CType::Void
                    }
                }
            }
            UnaryOp::AddrOf | UnaryOp::AddrOfMut => {
                if !is_place(operand) {
                    self.errors.push(TypeError::InvalidAddrOf {
                        span: span.clone(),
                    });
                }
                let ty = self.check_expr(operand, None);
                CType::Pointer {
                    pointee: Box::new(ty),
                    mutable: op == UnaryOp::AddrOfMut,
                }
            }
        }
    }

    fn binary_op_method_name(op: BinaryOp) -> Option<&'static str> {
        Some(match op {
            BinaryOp::Add => "add",
            BinaryOp::Sub => "sub",
            BinaryOp::Mul => "mul",
            BinaryOp::Div => "div",
            BinaryOp::Mod => "mod",
            BinaryOp::Eq => "eq",
            BinaryOp::NotEq => "ne",
            BinaryOp::Lt => "lt",
            BinaryOp::Gt => "gt",
            BinaryOp::LtEq => "le",
            BinaryOp::GtEq => "ge",
            BinaryOp::AndAnd => "and",
            BinaryOp::OrOr => "or",
            BinaryOp::BitAnd => "bit_and",
            BinaryOp::BitOr => "bit_or",
            BinaryOp::BitXor => "bit_xor",
            BinaryOp::Shl => "shl",
            BinaryOp::Shr => "shr",
        })
    }

    fn check_binary(&mut self, op: BinaryOp, lhs: &Expr, rhs: &Expr, span: &SourceSpan) -> CType {
        use BinaryOp::*;
        if let Some(method_name) = Self::binary_op_method_name(op) {
            let lhs_ty = self.check_expr(lhs, None);
            let receiver = match &lhs_ty {
                CType::Struct { name, .. } => Some(name.clone()),
                CType::Pointer { pointee, .. } => match &**pointee {
                    CType::Struct { name, .. } => Some(name.clone()),
                    _ => None,
                },
                _ => None,
            };
            if let Some(struct_name) = receiver {
                let overloads = self.methods.get(&struct_name).and_then(|m| m.get(method_name)).cloned().unwrap_or_default();
                if !overloads.is_empty() {
                    return self.check_method_call(lhs, method_name, &[rhs.clone()], span);
                }
            }
        }
        match op {
            AndAnd | OrOr => {
                let l = self.check_expr(lhs, None);
                if !matches!(l, CType::Bool) {
                    self.errors.push(TypeError::NonBoolOperand {
                        span: span.clone(),
                        op: if op == AndAnd { "&&" } else { "||" },
                        found: l.to_string(),
                    });
                }
                let r = self.check_expr(rhs, None);
                if !matches!(r, CType::Bool) {
                    self.errors.push(TypeError::NonBoolOperand {
                        span: span.clone(),
                        op: if op == AndAnd { "&&" } else { "||" },
                        found: r.to_string(),
                    });
                }
                CType::Bool
            }
            Eq | NotEq | Lt | Gt | LtEq | GtEq => {
                let l = self.check_expr(lhs, None);
                let r = self.check_expr(rhs, Some(&l));
                let unified = match CType::unify(&l, &r) {
                    Some(u) => u,
                    None => {
                        self.errors.push(TypeError::TypeMismatch {
                            span: span.clone(),
                            expected: l.to_string(),
                            found: r.to_string(),
                        });
                        return CType::Bool;
                    }
                };
                if matches!(op, Lt | Gt | LtEq | GtEq) && !unified.accepts_numeric() {
                    self.errors.push(TypeError::NonNumericOperand {
                        span: span.clone(),
                        op: op_str(op),
                        found: unified.to_string(),
                    });
                }
                CType::Bool
            }
            Add | Sub | Mul | Div | Mod | BitAnd | BitOr | BitXor | Shl | Shr => {
                let l = self.check_expr(lhs, None);
                let r = self.check_expr(rhs, Some(&l));
                let unified = match CType::unify(&l, &r) {
                    Some(u) => u,
                    None => {
                        self.errors.push(TypeError::TypeMismatch {
                            span: span.clone(),
                            expected: l.to_string(),
                            found: r.to_string(),
                        });
                        return l;
                    }
                };
                match op {
                    BitAnd | BitOr | BitXor | Shl | Shr => {
                        if !unified.accepts_int() {
                            self.errors.push(TypeError::NonIntegerOperand {
                                span: span.clone(),
                                op: op_str(op),
                                found: unified.to_string(),
                            });
                        }
                    }
                    _ => {
                        if !unified.accepts_numeric() {
                            self.errors.push(TypeError::NonNumericOperand {
                                span: span.clone(),
                                op: op_str(op),
                                found: unified.to_string(),
                            });
                        }
                    }
                }
                unified
            }
        }
    }

    fn check_assign(&mut self, target: &Expr, op: AssignOp, value: &Expr, span: &SourceSpan) -> CType {
        let target_ty = self.check_expr(target, None);
        match target {
            Expr::Var { name, .. } => match self.scopes.lookup(name) {
                Some(info) if !info.mutable => {
                    self.errors.push(TypeError::AssignToImmutable {
                        span: span.clone(),
                        name: name.clone(),
                    });
                }
                _ => {}
            },
            Expr::Unary { op: UnaryOp::Deref, operand, .. } => {
                let ptr_ty = self.check_expr(operand, None);
                if let CType::Pointer { mutable: false, .. } = &ptr_ty {
                    self.errors.push(TypeError::AssignThroughConstPointer {
                        span: span.clone(),
                    });
                }
            }
            Expr::FieldAccess { .. } | Expr::Index { .. } => {}
            _ => {
                self.errors.push(TypeError::InvalidAssignTarget {
                    span: span.clone(),
                });
            }
        }
        if op != AssignOp::Assign {
            match op {
                AssignOp::Mod => {
                    if !target_ty.accepts_int() {
                        self.errors.push(TypeError::NonIntegerOperand {
                            span: span.clone(),
                            op: "%=",
                            found: target_ty.to_string(),
                        });
                    }
                }
                _ => {
                    if !target_ty.accepts_numeric() {
                        self.errors.push(TypeError::NonNumericOperand {
                            span: span.clone(),
                            op: assign_op_str(op),
                            found: target_ty.to_string(),
                        });
                    }
                }
            }
        }
        let value_ty = self.check_expr(value, Some(&target_ty));
        if CType::unify(&value_ty, &target_ty).is_none() {
            self.errors.push(TypeError::TypeMismatch {
                span: value.span().clone(),
                expected: target_ty.to_string(),
                found: value_ty.to_string(),
            });
        }
        target_ty
    }

    fn check_cast(&mut self, inner: &Expr, ty: &Type, span: &SourceSpan) -> CType {
        let from = self.check_expr(inner, None);
        let to = self.convert_type(ty);
        let castable = |t: &CType| {
            t.is_numeric()
                || matches!(t, CType::Char | CType::IntLiteral | CType::FloatLiteral)
        };
        let compatible = (castable(&from) && castable(&to))
            || (matches!(from, CType::Pointer { .. } | CType::Str)
                && matches!(to, CType::Pointer { .. }));
        if !compatible {
            self.errors.push(TypeError::UnsupportedCast {
                span: span.clone(),
                from: from.to_string(),
                to: to.to_string(),
            });
        }
        to
    }

    fn check_struct_literal(
        &mut self,
        name: &str,
        args: &[Type],
        fields: &[(String, Expr)],
        span: &SourceSpan,
        expected: Option<&CType>,
    ) -> CType {
        let Some(info) = self.structs.get(name).cloned() else {
            self.errors.push(TypeError::UndefinedType {
                span: span.clone(),
                name: name.to_string(),
            });
            return CType::Void;
        };
        let resolved_args = if info.generics.is_empty() {
            if !args.is_empty() {
                self.errors.push(TypeError::NotGeneric {
                    span: span.clone(),
                    name: name.to_string(),
                });
                return CType::Void;
            }
            Vec::new()
        } else {
            if !args.is_empty() {
                if args.len() != info.generics.len() {
                    self.errors.push(TypeError::GenericArgCount {
                        span: span.clone(),
                        name: name.to_string(),
                        expected: info.generics.len(),
                        found: args.len(),
                    });
                    return CType::Void;
                }
                args.iter().map(|a| self.convert_type(a)).collect()
            } else if let Some(CType::Struct { name: exp_name, args: exp_args }) = expected {
                if *exp_name == name {
                    exp_args.clone()
                } else {
                    self.errors.push(TypeError::RequiresGenericArgs {
                        span: span.clone(),
                        name: name.to_string(),
                    });
                    return CType::Void;
                }
            } else {
                self.errors.push(TypeError::RequiresGenericArgs {
                    span: span.clone(),
                    name: name.to_string(),
                });
                return CType::Void;
            }
        };
        let bindings: HashMap<String, CType> = info
            .generics
            .iter()
            .zip(resolved_args.iter())
            .map(|(g, ty)| (g.clone(), ty.clone()))
            .collect();
        let mut seen = HashMap::new();
        for (field_name, value) in fields {
            if seen.insert(field_name.clone(), ()).is_some() {
                self.errors.push(TypeError::DuplicateFieldInLiteral {
                    span: value.span().clone(),
                    name: field_name.clone(),
                });
                continue;
            }
            match info.fields.iter().find(|(f, _)| f == field_name) {
                Some((_, ty)) => {
                    self.check_expr(value, Some(&ty.substitute(&bindings)));
                }
                None => {
                    self.errors.push(TypeError::UnknownField {
                        span: value.span().clone(),
                        name: field_name.clone(),
                        ty: name.to_string(),
                    });
                }
            }
        }
        for (field_name, _) in &info.fields {
            if !seen.contains_key(field_name) {
                self.errors.push(TypeError::MissingField {
                    span: span.clone(),
                    name: name.to_string(),
                    field: field_name.clone(),
                });
            }
        }
        CType::Struct {
            name: name.to_string(),
            args: resolved_args,
        }
    }

    fn check_match(
        &mut self,
        scrutinee: &Expr,
        arms: &[MatchArm],
        span: &SourceSpan,
        expected: Option<&CType>,
    ) -> CType {
        let scrutinee_ty = self.check_expr(scrutinee, None);
        let is_bool = matches!(scrutinee_ty, CType::Bool);
        let mut has_catch_all = false;
        let mut has_true = false;
        let mut has_false = false;
        for arm in arms {
            match &arm.pattern {
                
                Pattern::Wildcard | Pattern::Binding(_) => has_catch_all = true,
                Pattern::Literal(LiteralExpr::Bool(true)) => has_true = true,
                Pattern::Literal(LiteralExpr::Bool(false)) => has_false = true,
                _ => {}
            }
        }
        if is_bool {
            if !has_catch_all && !(has_true && has_false) {
                self.errors.push(TypeError::NonExhaustiveMatch {
                    span: span.clone(),
                });
            }
        } else if !has_catch_all {
            self.errors.push(TypeError::NonExhaustiveMatch {
                span: span.clone(),
            });
        }

        let mut result_ty = CType::Void;
        for arm in arms {
            self.scopes.push_scope();
            match &arm.pattern {
                Pattern::Wildcard => {}
                Pattern::Literal(lit) => {
                    let pat_ty = literal_type(lit, Some(&scrutinee_ty));
                    if CType::unify(&pat_ty, &scrutinee_ty).is_none() {
                        self.errors.push(TypeError::PatternMismatch {
                            span: arm.span.clone(),
                            expected: scrutinee_ty.to_string(),
                            found: pat_ty.to_string(),
                        });
                    }
                }
                Pattern::Binding(name) => {
                    self.scopes.define(
                        name.clone(),
                        VarInfo {
                            ty: scrutinee_ty.clone(),
                            mutable: false,
                        },
                    );
                }
            }
            match &arm.body {
                MatchArmBody::Expr(e) => {
                    result_ty = self.check_expr(e, expected);
                }
                MatchArmBody::Block(b) => self.check_block(b),
            }
            self.scopes.pop_scope();
        }
        result_ty
    }

    fn check_return(&mut self, value: Option<&Expr>, span: &SourceSpan) -> CType {
        let return_ty = self.return_ty.clone();
        match (value, &return_ty) {
            (None, Some(expected)) => {
                self.errors.push(TypeError::MissingReturnValue {
                    span: span.clone(),
                    expected: expected.to_string(),
                });
            }
            (Some(_), None) => {
                self.errors.push(TypeError::VoidReturnValue {
                    span: span.clone(),
                });
            }
            (Some(v), Some(expected)) => {
                let found = self.check_expr(v, Some(expected));
                if CType::unify(&found, expected).is_none() {
                    self.errors.push(TypeError::ReturnMismatch {
                        span: v.span().clone(),
                        expected: expected.to_string(),
                        found: found.to_string(),
                    });
                }
            }
            (None, None) => {}
        }
        CType::Void
    }

    fn check_range(
        &mut self,
        start: &Expr,
        end: &Expr,
        _inclusive: bool,
        _span: &SourceSpan,
    ) -> CType {
        let start_ty = self.check_expr(start, None);
        let end_ty = self.check_expr(end, None);
        if !start_ty.is_numeric() && !matches!(start_ty, CType::IntLiteral) {
            self.errors.push(TypeError::TypeMismatch {
                span: start.span().clone(),
                expected: "numeric type".to_string(),
                found: start_ty.to_string(),
            });
        }
        if !end_ty.is_numeric() && !matches!(end_ty, CType::IntLiteral) {
            self.errors.push(TypeError::TypeMismatch {
                span: end.span().clone(),
                expected: "numeric type".to_string(),
                found: end_ty.to_string(),
            });
        }
        CType::Void
    }

    fn check_array_repeat(
        &mut self,
        value: &Expr,
        count: &Expr,
        span: &SourceSpan,
        expected: Option<&CType>,
    ) -> CType {
        let elem_expected = match expected {
            Some(CType::Array { element, .. }) => Some(element.as_ref()),
            _ => None,
        };
        let val_ty = self.check_expr(value, elem_expected);
        match self.check_array_length(count, span) {
            Ok(len) => CType::Array {
                element: Box::new(val_ty),
                length: len,
            },
            Err(ty) => ty,
        }
    }

    fn check_array_literal(
        &mut self,
        elements: &[Expr],
        _span: &SourceSpan,
        expected: Option<&CType>,
    ) -> CType {
        let elem_expected = match expected {
            Some(CType::Array { element, .. }) => Some(element.as_ref()),
            _ => None,
        };
        let mut elem_ty = elem_expected.cloned().unwrap_or(CType::Void);
        for elem in elements {
            let ty = self.check_expr(elem, elem_expected);
            if elem_ty == CType::Void {
                elem_ty = ty;
            } else if CType::unify(&elem_ty, &ty).is_none() {
                self.errors.push(TypeError::TypeMismatch {
                    span: elem.span().clone(),
                    expected: elem_ty.to_string(),
                    found: ty.to_string(),
                });
            }
        }
        CType::Array {
            element: Box::new(elem_ty),
            length: elements.len() as u64,
        }
    }
}






fn literal_type(lit: &LiteralExpr, expected: Option<&CType>) -> CType {
    match lit {
        LiteralExpr::Integer(_) => match expected {
            Some(CType::I8)
            | Some(CType::I16)
            | Some(CType::I32)
            | Some(CType::I64)
            | Some(CType::I128)
            | Some(CType::U8)
            | Some(CType::U16)
            | Some(CType::U32)
            | Some(CType::U64)
            | Some(CType::U128)
            | Some(CType::Isize)
            | Some(CType::Usize) => expected.expect("checked above").clone(),
            _ => CType::IntLiteral,
        },
        LiteralExpr::Float(_) => match expected {
            Some(CType::F32) | Some(CType::F64) => expected.expect("checked above").clone(),
            _ => CType::FloatLiteral,
        },
        LiteralExpr::Char(_) => CType::Char,
        LiteralExpr::String(_) => CType::Str,
        LiteralExpr::Bool(_) => CType::Bool,
        LiteralExpr::Null => CType::Null,
    }
}


fn duplicates(names: &[String]) -> Vec<String> {
    let mut seen = HashMap::new();
    let mut dups = Vec::new();
    for name in names {
        if seen.insert(name, ()).is_some() && !dups.contains(name) {
            dups.push(name.clone());
        }
    }
    dups
}


fn dedup(names: &[String]) -> Vec<String> {
    let mut seen = HashMap::new();
    let mut out = Vec::new();
    for name in names {
        if seen.insert(name, ()).is_none() {
            out.push(name.clone());
        }
    }
    out
}


fn is_place(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Var { .. }
            | Expr::FieldAccess { .. }
            | Expr::Index { .. }
            | Expr::Unary { op: UnaryOp::Deref, .. }
    )
}

fn op_str(op: BinaryOp) -> &'static str {
    use BinaryOp::*;
    match op {
        Add => "+",
        Sub => "-",
        Mul => "*",
        Div => "/",
        Mod => "%",
        Eq => "==",
        NotEq => "!=",
        Lt => "<",
        Gt => ">",
        LtEq => "<=",
        GtEq => ">=",
        AndAnd => "&&",
        OrOr => "||",
        BitAnd => "&",
        BitOr => "|",
        BitXor => "^",
        Shl => "<<",
        Shr => ">>",
    }
}

fn assign_op_str(op: AssignOp) -> &'static str {
    use AssignOp::*;
    match op {
        Assign => "=",
        Add => "+=",
        Sub => "-=",
        Mul => "*=",
        Div => "/=",
        Mod => "%=",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    
    fn check(src: &str) -> Vec<TypeError> {
        let tokens = crate::lexer::lex(src, "test.cp").expect("lex should succeed");
        let program = crate::parser::parse_tokens(tokens).expect("parse should succeed");
        check_program(&program)
    }

    
    fn ok(src: &str) {
        let errors = check(src);
        assert!(errors.is_empty(), "expected no errors, got: {errors:?}");
    }

    
    fn err(src: &str, variant: fn(&TypeError) -> bool) {
        let errors = check(src);
        assert!(
            errors.iter().any(variant),
            "expected a matching error, got: {errors:?}"
        );
    }

    fn is_undefined_variable(e: &TypeError) -> bool {
        matches!(e, TypeError::UndefinedVariable { name, .. } if name == "y")
    }

    fn is_undefined_type(e: &TypeError) -> bool {
        matches!(e, TypeError::UndefinedType { .. })
    }

    fn is_undefined_function(e: &TypeError) -> bool {
        matches!(e, TypeError::UndefinedFunction { .. })
    }

    fn is_undefined_method(e: &TypeError) -> bool {
        matches!(e, TypeError::UndefinedMethod { .. })
    }

    fn is_type_mismatch(e: &TypeError) -> bool {
        matches!(e, TypeError::TypeMismatch { .. })
    }

    fn is_assign_to_immutable(e: &TypeError) -> bool {
        matches!(e, TypeError::AssignToImmutable { .. })
    }

    fn is_invalid_assign_target(e: &TypeError) -> bool {
        matches!(e, TypeError::InvalidAssignTarget { .. })
    }

    fn is_if_condition(e: &TypeError) -> bool {
        matches!(e, TypeError::IfConditionNotBool { .. })
    }

    fn is_while_condition(e: &TypeError) -> bool {
        matches!(e, TypeError::WhileConditionNotBool { .. })
    }

    fn is_non_exhaustive(e: &TypeError) -> bool {
        matches!(e, TypeError::NonExhaustiveMatch { .. })
    }

    fn is_arg_count(e: &TypeError) -> bool {
        matches!(e, TypeError::ArgCountMismatch { .. })
    }

    fn is_unknown_field_access(e: &TypeError) -> bool {
        matches!(e, TypeError::UnknownFieldAccess { .. })
    }

    fn is_missing_field(e: &TypeError) -> bool {
        matches!(e, TypeError::MissingField { .. })
    }

    fn is_unknown_field(e: &TypeError) -> bool {
        matches!(e, TypeError::UnknownField { .. })
    }

    fn is_duplicate_field_in_literal(e: &TypeError) -> bool {
        matches!(e, TypeError::DuplicateFieldInLiteral { .. })
    }

    fn is_duplicate_param(e: &TypeError) -> bool {
        matches!(e, TypeError::DuplicateParam { .. })
    }

    fn is_duplicate_struct_field(e: &TypeError) -> bool {
        matches!(e, TypeError::DuplicateStructField { .. })
    }

    fn is_assign_through_const(e: &TypeError) -> bool {
        matches!(e, TypeError::AssignThroughConstPointer { .. })
    }

    fn is_pattern_mismatch(e: &TypeError) -> bool {
        matches!(e, TypeError::PatternMismatch { .. })
    }

    fn is_return_mismatch(e: &TypeError) -> bool {
        matches!(e, TypeError::ReturnMismatch { .. })
    }

    fn is_missing_return_value(e: &TypeError) -> bool {
        matches!(e, TypeError::MissingReturnValue { .. })
    }

    fn is_void_return_value(e: &TypeError) -> bool {
        matches!(e, TypeError::VoidReturnValue { .. })
    }

    fn is_self_outside_method(e: &TypeError) -> bool {
        matches!(e, TypeError::SelfOutsideMethod { .. })
    }

    fn is_self_outside_impl(e: &TypeError) -> bool {
        matches!(e, TypeError::SelfOutsideImpl { .. })
    }

    fn is_generic_arg_count(e: &TypeError) -> bool {
        matches!(e, TypeError::GenericArgCount { .. })
    }

    fn is_not_generic(e: &TypeError) -> bool {
        matches!(e, TypeError::NotGeneric { .. })
    }

    fn is_non_integer(e: &TypeError) -> bool {
        matches!(e, TypeError::NonIntegerOperand { .. })
    }

    fn is_non_bool(e: &TypeError) -> bool {
        matches!(e, TypeError::NonBoolOperand { .. })
    }

    fn is_unsupported_cast(e: &TypeError) -> bool {
        matches!(e, TypeError::UnsupportedCast { .. })
    }

    fn is_not_iterable(e: &TypeError) -> bool {
        matches!(e, TypeError::NotIterable { .. })
    }

    fn is_private_member_access(e: &TypeError) -> bool {
        matches!(e, TypeError::PrivateMemberAccess { .. })
    }

    fn is_invalid_override(e: &TypeError) -> bool {
        matches!(e, TypeError::InvalidOverride { .. })
    }

    fn is_final_override(e: &TypeError) -> bool {
        matches!(e, TypeError::FinalOverride { .. })
    }

    
    
    

    #[test]
    fn valid_struct_program_passes() {
        ok(
            r#"
import std.io;

struct Vector2 { x: f64, y: f64 }

impl Vector2 {
    fn new(x: f64, y: f64) -> Vector2 { return Vector2 { x: x, y: y }; }
    fn length(self) -> f64 { return self.x + self.y; }
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
"#,
        );
    }

    #[test]
    fn valid_class_method_program_passes() {
        ok(
            r#"
class Vector2 {
    x: f64,
    y: f64,

    fn new(x: f64, y: f64) -> Vector2 {
        return Vector2 { x: x, y: y };
    }

    fn length(self) -> f64 {
        return self.x + self.y;
    }
}

fn main() -> i32 {
    let v: Vector2 = Vector2::new(1.0, 2.0);
    let l: f64 = v.length();
    println(l);
    return 0;
}
"#,
        );
    }

    #[test]
    fn valid_generic_class_method_program_passes() {
        ok(
            r#"
class Box[T] {
    value: T,

    fn new(v: T) -> Box[T] {
        return Box[T] { value: v };
    }

    fn get(self) -> T {
        return self.value;
    }
}

fn main() -> i32 {
    let b: Box[i32] = Box::new(5);
    let v: i32 = b.get();
    println(v);
    return 0;
}
"#,
        );
    }

    #[test]
    fn valid_constructor_and_cleanup_style_methods_passes() {
        ok(
            r#"
struct Point {
    x: i32,
    y: i32,
}

impl Point {
    fn new(x: i32, y: i32) -> Point {
        return Point { x: x, y: y };
    }

    fn destroy(self) -> i32 {
        return self.x + self.y;
    }
}

fn main() -> i32 {
    let p: Point = Point(3, 4);
    let total: i32 = p.destroy();
    return total;
}
"#,
        );
    }

    #[test]
    fn valid_operator_overload_program_passes() {
        ok(
            r#"
struct Vec2 {
    x: i32,
    y: i32,
}

impl Vec2 {
    fn add(self, rhs: Vec2) -> Vec2 {
        return Vec2 { x: self.x + rhs.x, y: self.y + rhs.y };
    }

    fn eq(self, rhs: Vec2) -> bool {
        return self.x == rhs.x && self.y == rhs.y;
    }
}

fn main() -> i32 {
    let a: Vec2 = Vec2 { x: 1, y: 2 };
    let b: Vec2 = Vec2 { x: 3, y: 4 };
    let c: Vec2 = a + b;
    if a == b {
        return 0;
    }
    return 1;
}
"#,
        );
    }

    #[test]
    fn valid_inherited_struct_fields_and_methods_pass() {
        ok(
            r#"
struct Base {
    x: i32,
}

struct Derived : Base {
    y: i32,
}

impl Derived {
    fn new(x: i32, y: i32) -> Derived {
        return Derived { x: x, y: y };
    }
}

fn main() -> i32 {
    let d: Derived = Derived::new(7, 9);
    return d.x + d.y;
}
"#,
        );
    }

    #[test]
    fn valid_visibility_and_virtual_method_modifiers_pass() {
        ok(
            r#"
struct Base {
    pub value: i32,
}

impl Base {
    pub virtual fn compute(self) -> i32 {
        return self.value;
    }
}

struct Derived : Base {
    pub extra: i32,
}

impl Derived {
    override fn compute(self) -> i32 {
        return self.value + self.extra;
    }
}

fn main() -> i32 {
    let d: Derived = Derived { value: 3, extra: 4 };
    return d.compute();
}
"#,
        );
    }

    #[test]
    fn valid_virtual_method_inheritance_chain_passes() {
        ok(
            r#"
struct Base {
    value: i32,
}

impl Base {
    pub virtual fn read(self) -> i32 {
        return self.value;
    }
}

struct Mid : Base {
    extra: i32,
}

struct Derived : Mid {
    bonus: i32,
}

impl Derived {
    override fn read(self) -> i32 {
        return self.value + self.extra + self.bonus;
    }
}

fn main() -> i32 {
    let d: Derived = Derived { value: 1, extra: 2, bonus: 3 };
    return d.read();
}
"#,
        );
    }

    #[test]
    fn protected_field_access_outside_inheritance_is_rejected() {
        err(
            r#"
struct Base {
    protected x: i32,
}

fn main() -> i32 {
    let b: Base = Base { x: 7 };
    return b.x;
}
"#,
            is_private_member_access,
        );
    }

    #[test]
    fn private_field_access_outside_struct_is_rejected() {
        err(
            r#"
struct Widget {
    private secret: i32,
}

fn main() -> i32 {
    let w: Widget = Widget { secret: 7 };
    return w.secret;
}
"#,
            is_private_member_access,
        );
    }

    #[test]
    fn override_requires_base_virtual_method() {
        err(
            r#"
struct Base {
    value: i32,
}

struct Derived : Base {
    fn override_value(self) -> i32 {
        return 0;
    }
}

impl Derived {
    override fn value(self) -> i32 {
        return 1;
    }
}

fn main() -> i32 {
    return 0;
}
"#,
            is_invalid_override,
        );
    }

    #[test]
    fn final_class_and_final_method_are_rejected_for_override() {
        err(
            r#"
final struct Base {
    value: i32,
}

impl Base {
    final fn read(self) -> i32 {
        return self.value;
    }
}

struct Derived : Base {
    extra: i32,
}

impl Derived {
    override fn read(self) -> i32 {
        return self.value + self.extra;
    }
}

fn main() -> i32 {
    let d: Derived = Derived { value: 1, extra: 2 };
    return d.read();
}
"#,
            is_final_override,
        );
    }

    #[test]
    fn abstract_class_and_method_are_supported() {
        ok(
            r#"
abstract struct Shape {
    x: i32,
    y: i32,
}

impl Shape {
    abstract fn area(self) -> i32;
}

struct Circle : Shape {
    radius: i32,
}

impl Circle {
    override fn area(self) -> i32 {
        return self.radius * self.radius;
    }
}

fn main() -> i32 {
    let c: Circle = Circle { x: 0, y: 0, radius: 5 };
    return c.area();
}
"#,
        );
    }

    #[test]
    fn valid_static_method_syntax_and_call_passes() {
        ok(
            r#"
struct Counter {
    value: i32,
}

impl Counter {
    static fn from(value: i32) -> Counter {
        return Counter { value: value };
    }
}

fn main() -> i32 {
    let c: Counter = Counter::from(7);
    return c.value;
}
"#,
        );
    }

    #[test]
    fn valid_generic_function_passes() {
        ok(
            r#"
fn classify[T](value: T) -> i32 {
    let mut result: i32 = 0;
    match value {
        1 => result = 10,
        _ => result = -1,
    }
    return result;
}
"#,
        );
    }

    #[test]
    fn valid_reference_and_namespace_program_passes() {
        ok(
            r#"
namespace math {
    fn add(a: i32, b: i32) -> i32 {
        return a + b;
    }
}

fn main() -> i32 {
    let mut x: i32 = 10;
    let r: &mut i32 = &mut x;
    *r = math::add(*r, 5);
    return *r;
}
"#,
        );
    }

    #[test]
    fn valid_overloaded_functions_and_methods_passes() {
        ok(
            r#"
fn add(a: i32) -> i32 {
    return a;
}

fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

struct Box {
    value: i32,
}

impl Box {
    fn scale(self) -> i32 {
        return self.value;
    }

    fn scale(self, amount: i32) -> i32 {
        return self.value * amount;
    }
}

fn main() -> i32 {
    let a: i32 = add(1);
    let b: i32 = add(1, 2);
    let box_value: Box = Box { value: 5 };
    let c: i32 = box_value.scale();
    let d: i32 = box_value.scale(3);
    println(a);
    println(b);
    println(c);
    println(d);
    return a + b + c + d;
}
"#,
        );
    }

    #[test]
    fn valid_pointer_program_passes() {
        ok(
            r#"
struct S { a: i32 }
fn read(p: *const S) -> i32 { return p.a; }
fn write(p: *mut i32) { *p = 5; }
fn idx(p: *const i32) -> i32 { return p[0]; }
"#,
        );
    }

    #[test]
    fn valid_self_return_type_in_impl() {
        ok(
            r#"
struct S { a: i32 }
impl S {
    fn make() -> Self { return S { a: 1 }; }
    fn as_ptr(self) -> *const Self { return &self; }
}
"#,
        );
    }

    #[test]
    fn valid_borrowed_self_method_receivers_passes() {
        ok(
            r#"
struct Counter {
    value: i32,
}

impl Counter {
    fn read(&self) -> i32 {
        return self.value;
    }

    fn add(&mut self, amount: i32) -> i32 {
        self.value = self.value + amount;
        return self.value;
    }
}

fn main() -> i32 {
    let mut c: Counter = Counter { value: 2 };
    let a: i32 = c.read();
    let b: i32 = c.add(3);
    return a + b;
}
"#,
        );
    }

    #[test]
    fn valid_const_method_declaration_passes() {
        ok(
            r#"
struct Counter {
    value: i32,
}

impl Counter {
    fn read(&self) const -> i32 {
        return self.value;
    }
}

fn main() -> i32 {
    let c: Counter = Counter { value: 7 };
    return c.read();
}
"#,
        );
    }

    #[test]
    fn method_call_on_pointer_receiver_auto_derefs() {
        ok(
            r#"
struct S { a: i32 }
impl S { fn get(self) -> i32 { return self.a; } }
fn f(p: *mut S) -> i32 { return p.get(); }
"#,
        );
    }

    #[test]
    fn valid_scopes_shadow_inner_block() {
        ok("fn f() { let x: i32 = 1; { let x: i32 = 2; } return; }");
    }

    #[test]
    fn module_qualified_builtin_passes() {
        ok("import std.io; fn f() { std::io::println(42); }");
    }

    #[test]
    fn valid_casts_and_literal_inference() {
        ok(
            r#"
fn f() {
    let a: i64 = 1 + 2;
    let b: f64 = 1.5 * 2.0;
    let c: i32 = 1;
    let d: f64 = c as f64;
    let e: f64 = -1.5;
}
"#,
        );
    }

    #[test]
    fn valid_match_on_bool_is_exhaustive_with_all_patterns() {
        ok(
            r#"
fn f(b: bool) -> i32 {
    match b {
        true => 1,
        false => 0,
    }
}
"#,
        );
    }

    #[test]
    fn for_loop_over_pointer_passes() {
        ok("fn f(p: *const i32) { for i in p { println(i); } }");
    }

    
    
    

    #[test]
    fn undefined_variable() {
        err("fn f() { let x = y; }", is_undefined_variable);
    }

    #[test]
    fn undefined_variable_in_assignment() {
        err("fn f() { zzz = 1; }", |e| {
            matches!(e, TypeError::UndefinedVariable { name, .. } if name == "zzz")
        });
    }

    #[test]
    fn undefined_type() {
        err("fn f(x: Foo) {}", is_undefined_type);
    }

    #[test]
    fn undefined_function() {
        err("fn f() { frobnicate(); }", is_undefined_function);
    }

    #[test]
    fn undefined_method() {
        err(
            "struct S { a: i32 } impl S { fn m(self) {} } fn f() { let s: S = S { a: 1 }; s.zzz(); }",
            is_undefined_method,
        );
    }

    #[test]
    fn calling_a_type_is_an_error() {
        err("struct S { a: i32 } fn f() { let s = S { a: 1 }; let t = s; S(t); }", |e| {
            matches!(e, TypeError::TypeAsValue { name, .. } if name == "S")
        });
    }

    #[test]
    fn multiple_errors_are_collected() {
        let errors = check("fn f() { let x = y; let a: Foo = b; }");
        assert!(errors.len() >= 2, "expected multiple errors, got: {errors:?}");
    }

    
    
    

    #[test]
    fn type_mismatch_in_let() {
        err("fn f() { let x: i32 = true; }", is_type_mismatch);
    }

    #[test]
    fn type_mismatch_in_return() {
        err("fn f() -> i32 { return true; }", is_type_mismatch);
    }

    #[test]
    fn return_mismatch_from_variable() {
        err("fn f() -> i32 { let b: bool = true; return b; }", is_return_mismatch);
    }

    #[test]
    fn return_without_value_in_typed_fn() {
        err("fn f() -> i32 { return; }", is_missing_return_value);
    }

    #[test]
    fn return_value_in_void_fn() {
        err("fn f() { return 1; }", is_void_return_value);
    }

    #[test]
    fn arithmetic_on_mismatched_types() {
        err("fn f() { let x: f64 = 1.0; let y: i32 = 1; let z: f64 = x + y; }", is_type_mismatch);
    }

    #[test]
    fn field_type_mismatch_in_struct_literal() {
        err("struct S { a: i32 } fn f() { let s: S = S { a: true }; }", is_type_mismatch);
    }

    #[test]
    fn call_argument_type_mismatch() {
        err("fn g(x: i32) {} fn f() { g(true); }", is_type_mismatch);
    }

    
    
    

    #[test]
    fn assign_to_immutable_is_error() {
        err("fn f() { let x: i32 = 1; x = 2; }", is_assign_to_immutable);
    }

    #[test]
    fn assign_to_literal_is_error() {
        err("fn f() { 5 = 3; }", is_invalid_assign_target);
    }

    #[test]
    fn assign_through_const_pointer_is_error() {
        err("fn f(p: *const i32) { *p = 5; }", is_assign_through_const);
    }

    #[test]
    fn mutable_variable_allows_assignment() {
        ok("fn f() { let mut x: i32 = 1; x = 2; x += 3; }");
    }

    
    
    

    #[test]
    fn if_condition_must_be_bool() {
        err("fn f() { if 1 { } }", is_if_condition);
    }

    #[test]
    fn while_condition_must_be_bool() {
        err("fn f() { while 1 { } }", is_while_condition);
    }

    #[test]
    fn bool_conditions_pass() {
        ok("fn f(b: bool) { if b { } while b { } }");
    }

    
    
    

    #[test]
    fn bool_match_requires_full_coverage() {
        err("fn f(b: bool) { match b { true => 1 } }", is_non_exhaustive);
    }

    #[test]
    fn non_bool_match_requires_wildcard() {
        err("fn f(x: i32) { match x { 1 => 1, 2 => 2 } }", is_non_exhaustive);
    }

    #[test]
    fn pattern_type_mismatch() {
        err("fn f(x: i32) { match x { true => 1, _ => 0 } }", is_pattern_mismatch);
    }

    #[test]
    fn match_binding_binds_variable() {
        ok("fn f(x: i32) { match x { v => println(v), } }");
    }

    
    
    

    #[test]
    fn call_arg_count_mismatch() {
        err("fn g(a: i32, b: i32) {} fn f() { g(1); }", is_arg_count);
    }

    #[test]
    fn method_requires_receiver() {
        err(
            "struct S { a: i32 } impl S { fn m(self) {} } fn f() { S::m(); }",
            |e| matches!(e, TypeError::MethodRequiresReceiver { .. }),
        );
    }

    #[test]
    fn associated_function_call_without_receiver_passes() {
        ok(
            "struct S { a: i32 } impl S { fn make() -> S { return S { a: 1 }; } } fn f() { let s: S = S::make(); }",
        );
    }

    #[test]
    fn undefined_builtin_bare_println_needs_arg() {
        
        err("fn f() { println(); }", is_arg_count);
    }

    
    
    

    #[test]
    fn unknown_field_access() {
        err(
            "struct S { a: i32 } fn f() { let s: S = S { a: 1 }; let x: i32 = s.b; }",
            is_unknown_field_access,
        );
    }

    #[test]
    fn field_access_on_non_struct() {
        err("fn f(x: i32) { let y: i32 = x.a; }", |e| {
            matches!(e, TypeError::NotAStruct { .. })
        });
    }

    #[test]
    fn struct_literal_missing_field() {
        err("struct S { a: i32, b: i32 } fn f() { let s: S = S { a: 1 }; }", is_missing_field);
    }

    #[test]
    fn struct_literal_unknown_field() {
        err("struct S { a: i32 } fn f() { let s: S = S { zz: 1 }; }", is_unknown_field);
    }

    #[test]
    fn struct_literal_duplicate_field() {
        err(
            "struct S { a: i32 } fn f() { let s: S = S { a: 1, a: 2 }; }",
            is_duplicate_field_in_literal,
        );
    }

    #[test]
    fn struct_literal_undefined_type() {
        err("fn f() { let s: Nope = Nope { a: 1 }; }", is_undefined_type);
    }

    
    
    

    #[test]
    fn duplicate_params() {
        err("fn f(a: i32, a: i32) {}", is_duplicate_param);
    }

    #[test]
    fn duplicate_struct_fields() {
        err("struct S { a: i32, a: f64 }", is_duplicate_struct_field);
    }

    #[test]
    fn let_without_type_or_init() {
        err("fn f() { let x; }", |e| matches!(e, TypeError::MissingLetType { .. }));
    }

    #[test]
    fn shadowing_in_same_scope_is_error() {
        err("fn f() { let x: i32 = 1; let x: i32 = 2; }", |e| {
            matches!(e, TypeError::AlreadyDefined { .. })
        });
    }

    
    
    

    #[test]
    fn self_outside_method() {
        err("fn f() { let x = self; }", is_self_outside_method);
    }

    #[test]
    fn self_type_outside_impl() {
        err("fn f() -> Self { }", is_self_outside_impl);
    }

    #[test]
    fn self_reference_in_toplevel_fn_call() {
        err("fn f() { Self::make(); }", is_self_outside_impl);
    }

    
    
    

    #[test]
    fn generic_arg_count_mismatch() {
        err("struct Opt[T] { v: T } fn f() { let o: Opt[i32, f64]; }", is_generic_arg_count);
    }

    #[test]
    fn generic_with_correct_args_passes() {
        ok("struct Opt[T] { v: T } fn f(o: Opt[i32]) {}");
    }

    #[test]
    fn non_generic_used_with_args() {
        err("struct S { a: i32 } fn f() { let s: S[i32]; }", is_not_generic);
    }

    #[test]
    fn generic_struct_without_args_is_error() {
        err("struct Opt[T] { v: T } fn f() { let o: Opt; }", |e| {
            matches!(e, TypeError::RequiresGenericArgs { .. })
        });
    }

    
    
    

    #[test]
    fn bitnot_on_float_is_error() {
        err("fn f() { let x: f64 = 1.0; let y: f64 = ~x; }", is_non_integer);
    }

    #[test]
    fn logical_and_on_ints_is_error() {
        err("fn f() { let x: i32 = 1; let b: bool = x && x; }", is_non_bool);
    }

    #[test]
    fn order_compare_on_bool_is_error() {
        err("fn f(b: bool) { let r: bool = b < b; }", |e| {
            matches!(e, TypeError::NonNumericOperand { .. })
        });
    }

    #[test]
    fn deref_non_pointer_is_error() {
        err("fn f(x: i32) { let y: i32 = *x; }", |e| {
            matches!(e, TypeError::DerefNonPointer { .. })
        });
    }

    #[test]
    fn address_of_non_place_is_error() {
        err("fn f() { let p = &5; }", |e| {
            matches!(e, TypeError::InvalidAddrOf { .. })
        });
    }

    #[test]
    fn unsupported_cast_is_error() {
        err("fn f() { let x: i32 = 1; let b: bool = x as bool; }", is_unsupported_cast);
    }

    
    
    

    #[test]
    fn index_non_pointer_is_error() {
        err("fn f(x: i32) { let y: i32 = x[0]; }", |e| {
            matches!(e, TypeError::NotIndexable { .. })
        });
    }

    #[test]
    fn index_must_be_integer() {
        err("fn f(p: *const i32, b: bool) { let y: i32 = p[b]; }", |e| {
            matches!(e, TypeError::IndexNotInteger { .. })
        });
    }

    #[test]
    fn for_over_non_iterable_is_error() {
        err("fn f() { let x: i32 = 1; for i in x { } }", is_not_iterable);
    }

    #[test]
    fn for_over_string_is_ok() {
        ok("fn f() { for c in \"abc\" { } }");
    }
}
