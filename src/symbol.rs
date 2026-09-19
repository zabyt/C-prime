








use std::collections::HashMap;

use crate::ast::PrimitiveType;







#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CType {
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
    
    Str,
    
    Null,
    
    Struct { name: String, args: Vec<CType> },
    
    Pointer { pointee: Box<CType>, mutable: bool },
    
    
    Generic(String),
    
    IntLiteral,
    
    FloatLiteral,
    
    Array { element: Box<CType>, length: u64 },
}

impl CType {
    
    pub fn from_primitive(p: PrimitiveType) -> CType {
        use PrimitiveType::*;
        match p {
            Void => CType::Void,
            Bool => CType::Bool,
            Char => CType::Char,
            I8 => CType::I8,
            I16 => CType::I16,
            I32 => CType::I32,
            I64 => CType::I64,
            I128 => CType::I128,
            U8 => CType::U8,
            U16 => CType::U16,
            U32 => CType::U32,
            U64 => CType::U64,
            U128 => CType::U128,
            F32 => CType::F32,
            F64 => CType::F64,
            Isize => CType::Isize,
            Usize => CType::Usize,
        }
    }

    
    pub fn is_int(&self) -> bool {
        matches!(
            self,
            CType::I8
                | CType::I16
                | CType::I32
                | CType::I64
                | CType::I128
                | CType::U8
                | CType::U16
                | CType::U32
                | CType::U64
                | CType::U128
                | CType::Isize
                | CType::Usize
        )
    }

    
    pub fn is_float(&self) -> bool {
        matches!(self, CType::F32 | CType::F64)
    }

    
    pub fn is_numeric(&self) -> bool {
        self.is_int() || self.is_float()
    }

    
    
    
    pub fn accepts_int(&self) -> bool {
        self.is_int() || matches!(self, CType::IntLiteral | CType::Generic(_))
    }

    
    pub fn accepts_float(&self) -> bool {
        self.is_float() || matches!(self, CType::FloatLiteral | CType::Generic(_))
    }

    
    pub fn accepts_numeric(&self) -> bool {
        self.accepts_int() || self.accepts_float()
    }

    
    
    
    
    
    
    pub fn unify(&self, other: &CType) -> Option<CType> {
        use CType::*;
        match (self, other) {
            (Generic(_), _) => Some(other.clone()),
            (_, Generic(_)) => Some(self.clone()),
            (IntLiteral, _) if other.is_int() => Some(other.clone()),
            (_, IntLiteral) if self.is_int() => Some(self.clone()),
            (FloatLiteral, _) if other.is_float() => Some(other.clone()),
            (_, FloatLiteral) if self.is_float() => Some(self.clone()),
            
            
            (Null, Pointer { .. }) => Some(other.clone()),
            (Pointer { .. }, Null) => Some(self.clone()),
            (Null, Str) => Some(other.clone()),
            (Str, Null) => Some(self.clone()),
            (Str, Pointer { pointee, mutable: _ }) if **pointee == CType::Char => Some(other.clone()),
            (Pointer { pointee, mutable: _ }, Str) if **pointee == CType::Char => Some(self.clone()),
            (Pointer { pointee, mutable }, Pointer { pointee: other_pointee, mutable: other_mut }) => {
                if *mutable != *other_mut {
                    return None;
                }
                pointee.unify(other_pointee).map(|inner| Pointer {
                    pointee: Box::new(inner),
                    mutable: *mutable,
                })
            }
            (Array { element, length }, Array { element: other_element, length: other_length }) => {
                if length != other_length {
                    return None;
                }
                element.unify(other_element).map(|inner| Array {
                    element: Box::new(inner),
                    length: *length,
                })
            }
            (Struct { name, args }, Struct { name: other_name, args: other_args }) => {
                if name != other_name || args.len() != other_args.len() {
                    return None;
                }
                let mut unified = Vec::with_capacity(args.len());
                for (a, b) in args.iter().zip(other_args.iter()) {
                    unified.push(a.unify(b)?);
                }
                Some(Struct {
                    name: name.clone(),
                    args: unified,
                })
            }
            _ if self == other => Some(self.clone()),
            _ => None,
        }
    }

    
    
    
    pub fn unify_arg(param: &CType, arg_ty: &CType) -> Option<CType> {
        if let Some(u) = param.unify(arg_ty) {
            return Some(u);
        }
        match (param, arg_ty) {
            
            
            (CType::Pointer { pointee: p, mutable: false }, CType::Pointer { pointee: a, mutable: true })
                if p.unify(a).is_some() =>
            {
                Some(arg_ty.clone())
            }
            (CType::Pointer { pointee, .. }, CType::Array { element, .. })
                if pointee.unify(element).is_some() =>
            {
                Some(arg_ty.clone())
            }
            _ => None,
        }
    }
}

impl std::fmt::Display for CType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use CType::*;
        match self {
            Void => f.write_str("void"),
            Bool => f.write_str("bool"),
            Char => f.write_str("char"),
            I8 => f.write_str("i8"),
            I16 => f.write_str("i16"),
            I32 => f.write_str("i32"),
            I64 => f.write_str("i64"),
            I128 => f.write_str("i128"),
            U8 => f.write_str("u8"),
            U16 => f.write_str("u16"),
            U32 => f.write_str("u32"),
            U64 => f.write_str("u64"),
            U128 => f.write_str("u128"),
            F32 => f.write_str("f32"),
            F64 => f.write_str("f64"),
            Isize => f.write_str("isize"),
            Usize => f.write_str("usize"),
            Str => f.write_str("str"),
            Null => f.write_str("null"),
            Struct { name, args } => {
                f.write_str(name)?;
                if !args.is_empty() {
                    write!(f, "[")?;
                    for (i, arg) in args.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{arg}")?;
                    }
                    write!(f, "]")?;
                }
                Ok(())
            }
            Pointer { pointee, mutable } => {
                if *mutable {
                    write!(f, "*mut {pointee}")
                } else {
                    write!(f, "*const {pointee}")
                }
            }
            Generic(name) => f.write_str(name),
            IntLiteral => f.write_str("<integer literal>"),
            FloatLiteral => f.write_str("<float literal>"),
            Array { element, length } => write!(f, "[{element}; {length}]"),
        }
    }
}

impl CType {
    
    
    
    pub fn substitute(&self, bindings: &HashMap<String, CType>) -> CType {
        use CType::*;
        match self {
            Generic(name) => bindings.get(name).cloned().unwrap_or_else(|| self.clone()),
            Struct { name, args } => Struct {
                name: name.clone(),
                args: args.iter().map(|a| a.substitute(bindings)).collect(),
            },
            Pointer { pointee, mutable } => Pointer {
                pointee: Box::new(pointee.substitute(bindings)),
                mutable: *mutable,
            },
            Array { element, length } => Array {
                element: Box::new(element.substitute(bindings)),
                length: *length,
            },
            other => other.clone(),
        }
    }

    
    
    pub fn contains_generic(&self) -> bool {
        use CType::*;
        match self {
            Generic(_) => true,
            Struct { args, .. } => args.iter().any(CType::contains_generic),
            Pointer { pointee, .. } => pointee.contains_generic(),
            Array { element, .. } => element.contains_generic(),
            _ => false,
        }
    }

    
    
    
    pub fn to_concrete(&self) -> CType {
        match self {
            CType::IntLiteral => CType::I64,
            CType::FloatLiteral => CType::F64,
            other => other.clone(),
        }
    }
}








pub fn match_type(
    pattern: &CType,
    other: &CType,
    bindings: &mut HashMap<String, CType>,
) -> bool {
    use CType::*;
    match pattern {
        Generic(name) => {
            let concrete = other.to_concrete();
            match bindings.get(name) {
                Some(bound) => bound == &concrete,
                None => {
                    bindings.insert(name.clone(), concrete);
                    true
                }
            }
        }
        Struct { name, args } => match other {
            Struct { name: oname, args: oargs } => {
                name == oname
                    && args.len() == oargs.len()
                    && args.iter().zip(oargs.iter()).all(|(p, a)| match_type(p, a, bindings))
            }
            _ => false,
        },
        Pointer { pointee, .. } => match other {
            Pointer { pointee: opointee, .. } => match_type(pointee, opointee, bindings),
            _ => false,
        },
        Array { element, length } => match other {
            Array { element: oelement, length: olength } => {
                length == olength && match_type(element, oelement, bindings)
            }
            _ => false,
        },
        _ => CType::unify(pattern, other).is_some(),
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarInfo {
    pub ty: CType,
    pub mutable: bool,
}


#[derive(Debug, Clone, Default)]
pub struct Scope {
    vars: HashMap<String, VarInfo>,
}

impl Scope {
    fn new() -> Scope {
        Scope::default()
    }

    fn define(&mut self, name: String, info: VarInfo) -> bool {
        if let std::collections::hash_map::Entry::Vacant(e) = self.vars.entry(name) {
            e.insert(info);
            true
        } else {
            false
        }
    }

    fn contains(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    fn lookup(&self, name: &str) -> Option<&VarInfo> {
        self.vars.get(name)
    }
}



#[derive(Debug, Clone, Default)]
pub struct ScopeManager {
    scopes: Vec<Scope>,
}

impl ScopeManager {
    
    pub fn new() -> ScopeManager {
        ScopeManager {
            scopes: vec![Scope::new()],
        }
    }

    
    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    
    
    pub fn define(&mut self, name: String, info: VarInfo) -> bool {
        self.scopes.last_mut().expect("scope stack is never empty").define(name, info)
    }

    
    pub fn is_in_current_scope(&self, name: &str) -> bool {
        self.scopes.last().expect("scope stack is never empty").contains(name)
    }

    
    pub fn lookup(&self, name: &str) -> Option<&VarInfo> {
        self.scopes.iter().rev().find_map(|scope| scope.lookup(name))
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructInfo {
    pub name: String,
    pub generics: Vec<String>,
    pub base: Option<String>,
    pub is_final: bool,
    pub is_abstract: bool,
    pub fields: Vec<(String, CType)>,
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FuncSig {
    pub name: String,
    pub generics: Vec<String>,
    pub params: Vec<CType>,
    
    pub has_self: bool,
    
    pub is_const: bool,
    
    pub is_virtual: bool,
    
    pub is_override: bool,
    
    pub is_final: bool,
    
    pub is_abstract: bool,
    
    pub is_static: bool,
    
    pub return_ty: Option<CType>,
    
    pub struct_name: Option<String>,
    
    
    pub is_builtin: bool,
    
    
    pub is_extern: bool,
    
    
    pub is_variadic: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn i32() -> CType {
        CType::I32
    }

    fn u64() -> CType {
        CType::U64
    }

    #[test]
    fn unify_identical_types() {
        assert_eq!(CType::unify(&i32(), &i32()), Some(i32()));
        assert_eq!(CType::unify(&CType::Bool, &CType::Bool), Some(CType::Bool));
    }

    #[test]
    fn unify_int_literal_with_integer_types() {
        assert_eq!(CType::unify(&CType::IntLiteral, &i32()), Some(i32()));
        assert_eq!(CType::unify(&i32(), &CType::IntLiteral), Some(i32()));
        assert_eq!(CType::unify(&CType::IntLiteral, &u64()), Some(u64()));
    }

    #[test]
    fn unify_float_literal_with_float_types() {
        assert_eq!(CType::unify(&CType::FloatLiteral, &CType::F64), Some(CType::F64));
        assert_eq!(CType::unify(&CType::F32, &CType::FloatLiteral), Some(CType::F32));
    }

    #[test]
    fn unify_rejects_mismatched_types() {
        assert_eq!(CType::unify(&i32(), &u64()), None);
        assert_eq!(CType::unify(&i32(), &CType::Bool), None);
        assert_eq!(CType::unify(&CType::IntLiteral, &CType::Bool), None);
        assert_eq!(CType::unify(&CType::FloatLiteral, &i32()), None);
    }

    #[test]
    fn unify_int_and_float_literals_is_rejected() {
        assert_eq!(CType::unify(&CType::IntLiteral, &CType::FloatLiteral), None);
    }

    #[test]
    fn unify_generic_with_anything() {
        let t = CType::Generic("T".to_string());
        assert_eq!(CType::unify(&t, &i32()), Some(i32()));
        assert_eq!(CType::unify(&i32(), &t), Some(i32()));
        assert_eq!(CType::unify(&t, &t), Some(t.clone()));
    }

    #[test]
    fn struct_types_unify_structurally() {
        let a = CType::Struct { name: "S".to_string(), args: vec![] };
        let b = CType::Struct { name: "S".to_string(), args: vec![] };
        let c = CType::Struct { name: "T".to_string(), args: vec![] };
        assert_eq!(CType::unify(&a, &b), Some(a.clone()));
        assert_eq!(CType::unify(&a, &c), None);
    }

    #[test]
    fn type_classification() {
        assert!(i32().is_int());
        assert!(CType::F64.is_float());
        assert!(CType::U8.is_numeric());
        assert!(!CType::Bool.is_numeric());
        assert!(!CType::Char.is_numeric());
        assert!(CType::IntLiteral.accepts_int());
        assert!(!CType::IntLiteral.is_int());
        assert!(CType::Generic("T".to_string()).accepts_numeric());
    }

    #[test]
    fn scope_defines_and_looks_up() {
        let mut scopes = ScopeManager::new();
        let info = VarInfo { ty: i32(), mutable: false };
        assert!(scopes.define("x".to_string(), info.clone()));
        assert_eq!(scopes.lookup("x"), Some(&info));
        assert!(scopes.lookup("y").is_none());
    }

    #[test]
    fn scope_rejects_duplicate_in_same_scope() {
        let mut scopes = ScopeManager::new();
        let info = VarInfo { ty: i32(), mutable: false };
        assert!(scopes.define("x".to_string(), info.clone()));
        assert!(!scopes.define("x".to_string(), info));
        assert!(scopes.is_in_current_scope("x"));
    }

    #[test]
    fn scope_shadowing_across_scopes() {
        let mut scopes = ScopeManager::new();
        let info = VarInfo { ty: i32(), mutable: false };
        scopes.define("x".to_string(), info.clone());
        scopes.push_scope();
        assert!(scopes.define("x".to_string(), VarInfo { ty: CType::Bool, mutable: true }));
        assert_eq!(scopes.lookup("x").unwrap().ty, CType::Bool);
        scopes.pop_scope();
        assert_eq!(scopes.lookup("x").unwrap().ty, i32());
    }

    #[test]
    fn scope_root_cannot_be_popped() {
        let mut scopes = ScopeManager::new();
        scopes.push_scope();
        scopes.push_scope();
        scopes.pop_scope();
        scopes.pop_scope();
        scopes.pop_scope();
        assert_eq!(scopes.lookup("nope"), None);
    }
}
