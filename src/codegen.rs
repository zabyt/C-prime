




















use std::collections::HashMap;
use std::fmt;
use std::path::Path;

use inkwell::values::{InstructionOpcode, InstructionValue};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetData, TargetMachine};
use inkwell::types::{BasicType, BasicTypeEnum, BasicMetadataTypeEnum, StructType};
use inkwell::values::{
    BasicMetadataValueEnum, BasicValue, BasicValueEnum, FunctionValue, IntValue, PointerValue,
};
use inkwell::{AddressSpace, FloatPredicate, IntPredicate, OptimizationLevel};

use crate::ast;
use crate::symbol::{match_type, CType, FuncSig, StructInfo};
use crate::token::SourceSpan;



#[derive(Debug, Clone)]
pub enum CodegenError {
    
    Unsupported { span: Option<SourceSpan>, message: String },
    
    Internal(String),
    
    Verify(String),
    
    Llvm(String),
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodegenError::Unsupported { span, message } => match span {
                Some(span) => write!(f, "{span}: {message}"),
                None => write!(f, "{message}"),
            },
            CodegenError::Internal(message) => write!(f, "internal codegen error: {message}"),
            CodegenError::Verify(message) => write!(f, "generated module failed verification: {message}"),
            CodegenError::Llvm(message) => write!(f, "LLVM error: {message}"),
        }
    }
}

impl std::error::Error for CodegenError {}

fn unsupported(span: &SourceSpan, message: impl Into<String>) -> CodegenError {
    CodegenError::Unsupported {
        span: Some(span.clone()),
        message: message.into(),
    }
}


pub fn emit_llvm_ir(program: &ast::Program) -> Result<String, CodegenError> {
    run(program, |_machine, cg| Ok(cg.module.to_string()))
}


pub fn emit_llvm_ir_to_file(program: &ast::Program, path: &Path) -> Result<(), CodegenError> {
    let ir = emit_llvm_ir(program)?;
    std::fs::write(path, ir).map_err(|err| CodegenError::Llvm(err.to_string()))
}


pub fn emit_object(program: &ast::Program, path: &Path) -> Result<(), CodegenError> {
    run(program, |machine, cg| {
        machine
            .write_to_file(&cg.module, FileType::Object, path)
            .map_err(|err| CodegenError::Llvm(err.to_string()))
    })
}



fn run<'ast, T>(
    program: &'ast ast::Program,
    f: impl FnOnce(&TargetMachine, &mut Codegen<'ast, '_>) -> Result<T, CodegenError>,
) -> Result<T, CodegenError> {
    let context = Context::create();
    let module = context.create_module("cprime");
    let builder = context.create_builder();

    
    
    Target::initialize_native(&InitializationConfig::default())
        .map_err(|err| CodegenError::Llvm(format!("failed to initialise native target: {err}")))?;
    let triple = TargetMachine::get_default_triple();
    let target = Target::from_triple(&triple).map_err(|err| CodegenError::Llvm(err.to_string()))?;
    let machine = target
        .create_target_machine(&triple, "generic", "", OptimizationLevel::None, RelocMode::PIC, CodeModel::Default)
        .ok_or_else(|| CodegenError::Llvm("failed to create a target machine".to_string()))?;

    module.set_triple(&triple);
    module.set_data_layout(&machine.get_target_data().get_data_layout());

    let mut cg = Codegen {
        context: &context,
        module,
        builder,
        data_layout: machine
            .get_target_data()
            .get_data_layout()
            .as_str()
            .to_str()
            .map_err(|err| CodegenError::Llvm(format!("bad data layout string: {err}")))?
            .to_string(),
        structs: HashMap::new(),
        enums: HashMap::new(),
        type_aliases: HashMap::new(),
        const_values: HashMap::new(),
        struct_types: HashMap::new(),
        signatures: HashMap::new(),
        methods: HashMap::new(),
        free_funcs: HashMap::new(),
        method_funcs: HashMap::new(),
        instance_sigs: HashMap::new(),
        free_ast: HashMap::new(),
        method_ast: HashMap::new(),
        pending: Vec::new(),
        var_stack: Vec::new(),
        current_struct: None,
        current_struct_args: Vec::new(),
        current_generics: HashMap::new(),
        current_fn: None,
        current_fn_return: CType::Void,
        string_globals: HashMap::new(),
        string_counter: 0,
        printf_fn: None,
        struct_literal_hint: None,
        control_flow: Vec::new(),
    };
    cg.gen_program(program)?;
    hoist_allocas(&cg.module, &context);

    cg.module.verify().map_err(|err| CodegenError::Verify(err.to_string()))?;
    f(&machine, &mut cg)
}






fn hoist_allocas(module: &Module<'_>, context: &Context) {
    let builder = context.create_builder();
    for function in module.get_functions() {
        let Some(entry) = function.get_first_basic_block() else {
            continue;
        };
        let allocas: Vec<InstructionValue<'_>> = function
            .get_basic_blocks()
            .into_iter()
            .flat_map(|block| block.get_instructions())
            .filter(|instruction| instruction.get_opcode() == InstructionOpcode::Alloca)
            .collect();
        for alloca in allocas {
            alloca.remove_from_basic_block();
            match entry.get_first_instruction() {
                Some(first) => builder.position_at(entry, &first),
                None => builder.position_at_end(entry),
            }
            builder.insert_instruction(&alloca, None);
        }
    }
}


struct Codegen<'ast, 'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    
    data_layout: String,
    
    structs: HashMap<String, StructInfo>,
    
    enums: HashMap<String, Vec<(String, Option<i128>)>>,
    
    type_aliases: HashMap<String, CType>,
    
    const_values: HashMap<String, &'ast ast::Expr>,
    
    
    struct_types: HashMap<String, StructType<'ctx>>,
    
    signatures: HashMap<String, FuncSig>,
    
    methods: HashMap<String, HashMap<String, FuncSig>>,
    
    free_funcs: HashMap<String, FunctionValue<'ctx>>,
    
    method_funcs: HashMap<String, HashMap<String, FunctionValue<'ctx>>>,
    
    instance_sigs: HashMap<String, FuncSig>,
    
    
    free_ast: HashMap<String, &'ast ast::Function>,
    
    method_ast: HashMap<String, HashMap<String, &'ast ast::Function>>,
    
    pending: Vec<PendingFunc<'ast>>,
    
    var_stack: Vec<HashMap<String, (PointerValue<'ctx>, CType)>>,
    
    current_struct: Option<String>,
    
    
    current_struct_args: Vec<CType>,
    
    
    current_generics: HashMap<String, CType>,
    
    current_fn: Option<FunctionValue<'ctx>>,
    
    current_fn_return: CType,
    
    string_globals: HashMap<String, PointerValue<'ctx>>,
    string_counter: u32,
    
    printf_fn: Option<FunctionValue<'ctx>>,
    
    
    
    
    struct_literal_hint: Option<(String, Vec<CType>)>,
    
    
    
    control_flow: Vec<ControlFlowTarget<'ctx>>,
}



#[derive(Clone, Copy)]
enum ControlFlowTarget<'ctx> {
    Loop {
        continue_bb: inkwell::basic_block::BasicBlock<'ctx>,
        exit_bb: inkwell::basic_block::BasicBlock<'ctx>,
    },
    Switch {
        exit_bb: inkwell::basic_block::BasicBlock<'ctx>,
    },
}


struct PendingFunc<'ast> {
    f: &'ast ast::Function,
    struct_name: Option<String>,
    
    struct_args: Vec<CType>,
    
    fn_args: Vec<CType>,
}

impl<'ctx, 'ast> Codegen<'ast, 'ctx> {
    
    
    

    fn gen_program(&mut self, program: &'ast ast::Program) -> Result<(), CodegenError> {
        
        for item in &program.items {
            match item {
                ast::Item::Struct(s) => self.collect_struct(s)?,
                ast::Item::Enum(e) => {
                    let mapped: Vec<(String, Option<i128>)> = e
                        .variants
                        .iter()
                        .map(|variant| {
                            let value = match &variant.value {
                                Some(ast::Expr::Literal(ast::LiteralExpr::Integer(v), _)) => Some(*v),
                                _ => None,
                            };
                            (variant.name.clone(), value)
                        })
                        .collect();
                    self.enums.insert(e.name.clone(), mapped);
                }
                ast::Item::TypeAlias(a) => {
                    self.type_aliases.insert(a.name.clone(), self.ctype_from_ast(&a.target, &a.span)?);
                }
                ast::Item::Const(c) => {
                    self.const_values.insert(c.name.clone(), &c.value);
                }
                _ => {}
            }
        }
        
        for item in &program.items {
            match item {
                ast::Item::Function(f) => self.collect_signature(f, None)?,
                ast::Item::Impl(b) => {
                    for method in &b.methods {
                        self.collect_signature(method, Some(&b.type_name))?;
                    }
                }
                ast::Item::ExternFunction(e) => self.collect_extern(e)?,
                _ => {}
            }
        }
        
        
        for item in &program.items {
            match item {
                ast::Item::Function(f) => {
                    if f.generics.is_empty() {
                        self.schedule_instance(f.name.clone(), &f.name, None, Vec::new(), Vec::new())?;
                    }
                }
                ast::Item::Impl(b) => {
                    let struct_generic = self.structs.get(&b.type_name).map(|i| !i.generics.is_empty()).unwrap_or(false);
                    for method in &b.methods {
                        if !struct_generic && method.generics.is_empty() {
                            let key = self.instance_key(&method.name, Some(&b.type_name), &[], &[]);
                            self.schedule_instance(key, &method.name, Some(&b.type_name), Vec::new(), Vec::new())?;
                        }
                    }
                }
                _ => {}
            }
        }
        
        let mut index = 0;
        while index < self.pending.len() {
            let task = PendingFunc {
                f: self.pending[index].f,
                struct_name: self.pending[index].struct_name.clone(),
                struct_args: self.pending[index].struct_args.clone(),
                fn_args: self.pending[index].fn_args.clone(),
            };
            self.gen_function(&task)?;
            index += 1;
        }
        Ok(())
    }

    fn collect_struct(&mut self, s: &'ast ast::StructDecl) -> Result<(), CodegenError> {
        let prev_gen = std::mem::take(&mut self.current_generics);
        
        
        for g in &s.generics {
            self.current_generics.insert(g.clone(), CType::Generic(g.clone()));
        }
        let mut fields = Vec::with_capacity(s.fields.len());
        if let Some(base) = &s.base {
            let base_type = match base {
                ast::Type::Named { name, args, .. } => {
                    if !args.is_empty() {
                        return Err(unsupported(&base.span(), format!("base type `{name}` has unsupported generic arguments")));
                    }
                    name.clone()
                }
                _ => return Err(unsupported(&base.span(), "struct inheritance requires a named base type")),
            };
            if let Some(base_info) = self.structs.get(&base_type) {
                fields.extend(base_info.fields.iter().cloned());
            }
        }
        for f in &s.fields {
            fields.push((f.name.clone(), self.ctype_from_ast(&f.ty, &f.span)?));
        }
        self.current_generics = prev_gen;
        self.structs.insert(
            s.name.clone(),
            StructInfo {
                name: s.name.clone(),
                generics: s.generics.clone(),
                base: s.base.as_ref().and_then(|base| match base {
                    ast::Type::Named { name, .. } => Some(name.clone()),
                    _ => None,
                }),
                is_final: s.is_final,
                is_abstract: s.is_abstract,
                fields,
            },
        );
        Ok(())
    }

    
    
    
    fn collect_signature(&mut self, f: &'ast ast::Function, struct_name: Option<&str>) -> Result<(), CodegenError> {
        let prev_struct = self.current_struct.take();
        self.current_struct = struct_name.map(str::to_string);
        let prev_args = std::mem::take(&mut self.current_struct_args);
        let prev_gen = std::mem::take(&mut self.current_generics);
        
        if let Some(s) = struct_name
            && let Some(info) = self.structs.get(s)
        {
            for g in &info.generics {
                self.current_generics.insert(g.clone(), CType::Generic(g.clone()));
            }
        }
        for g in &f.generics {
            self.current_generics.insert(g.clone(), CType::Generic(g.clone()));
        }
        let (param_ctys, return_cty) = self.function_types(f)?;
        self.current_struct = prev_struct;
        self.current_struct_args = prev_args;
        self.current_generics = prev_gen;

        let has_self = matches!(f.params.first(), Some(p) if p.name == "self");
        let sig = FuncSig {
            name: f.name.clone(),
            generics: f.generics.clone(),
            params: param_ctys,
            has_self,
            is_const: false,
            is_virtual: f.is_virtual || f.is_abstract,
            is_override: f.is_override,
            is_final: f.is_final,
            is_abstract: f.is_abstract,
            is_static: f.is_static,
            return_ty: if return_cty == CType::Void { None } else { Some(return_cty) },
            struct_name: struct_name.map(str::to_string),
            is_builtin: false,
            is_extern: false,
            is_variadic: false,
        };
        match struct_name {
            Some(s) => {
                self.methods.entry(s.to_string()).or_default().insert(f.name.clone(), sig);
                self.method_ast.entry(s.to_string()).or_default().insert(f.name.clone(), f);
            }
            None => {
                self.signatures.insert(f.name.clone(), sig);
                self.free_ast.insert(f.name.clone(), f);
            }
        }
        Ok(())
    }

    
    
    
    fn collect_extern(&mut self, e: &'ast ast::ExternFunctionDecl) -> Result<(), CodegenError> {
        let params = e
            .params
            .iter()
            .map(|p| self.ctype_from_ast(&p.ty, &p.span))
            .collect::<Result<Vec<_>, _>>()?;
        let return_ty = match &e.return_ty {
            Some(ty) => self.ctype_from_ast(ty, &e.span)?,
            None => CType::Void,
        };
        let sig = FuncSig {
            name: e.name.clone(),
            generics: Vec::new(),
            params: params.clone(),
            has_self: false,
            is_const: false,
            is_virtual: false,
            is_override: false,
            is_final: false,
            is_abstract: false,
            is_static: true,
            return_ty: if return_ty == CType::Void { None } else { Some(return_ty.clone()) },
            struct_name: None,
            is_builtin: false,
            is_extern: true,
            is_variadic: e.is_variadic,
        };
        let llvm_params = params
            .iter()
            .map(|ty| self.llvm_type(ty).map(BasicMetadataTypeEnum::from))
            .collect::<Result<Vec<_>, _>>()?;
        let vararg = e.is_variadic;
        let fn_ty = if return_ty == CType::Void {
            self.context.void_type().fn_type(&llvm_params, vararg)
        } else {
            self.llvm_type(&return_ty)?.fn_type(&llvm_params, vararg)
        };
        
        
        
        
        
        let fn_value = match self.module.get_function(&e.name) {
            Some(existing) => existing,
            None => {
                let added = self.module.add_function(&e.name, fn_ty, None);
                added.set_call_conventions(0);
                added
            }
        };
        self.signatures.insert(e.name.clone(), sig.clone());
        self.instance_sigs.insert(e.name.clone(), sig);
        self.free_funcs.insert(e.name.clone(), fn_value);
        Ok(())
    }

    
    fn resolve_generics(
        &self,
        f: &'ast ast::Function,
        struct_name: Option<&str>,
        struct_args: &[CType],
        fn_args: &[CType],
    ) -> Result<HashMap<String, CType>, CodegenError> {
        let mut map = HashMap::new();
        if let Some(s) = struct_name {
            let info = self
                .structs
                .get(s)
                .ok_or_else(|| CodegenError::Internal(format!("struct `{s}` was never collected")))?;
            for (g, ty) in info.generics.iter().zip(struct_args.iter()) {
                map.insert(g.clone(), ty.clone());
            }
        }
        for (g, ty) in f.generics.iter().zip(fn_args.iter()) {
            map.insert(g.clone(), ty.clone());
        }
        Ok(map)
    }

    
    fn lookup_instance(&self, struct_name: Option<&str>, key: &str) -> Option<FunctionValue<'ctx>> {
        match struct_name {
            Some(s) => self.method_funcs.get(s).and_then(|m| m.get(key)).copied(),
            None => self.free_funcs.get(key).copied(),
        }
    }

    
    fn instance_key(&self, name: &str, struct_name: Option<&str>, struct_args: &[CType], fn_args: &[CType]) -> String {
        let base = match struct_name {
            Some(s) => method_llvm_name(s, name),
            None => name.to_string(),
        };
        if struct_args.is_empty() && fn_args.is_empty() {
            return base;
        }
        let sa = mangle_args(struct_args);
        let fa = mangle_args(fn_args);
        if fa.is_empty() {
            format!("{base}$s{sa}")
        } else if sa.is_empty() {
            format!("{base}$f{fa}")
        } else {
            format!("{base}$s{sa}$f{fa}")
        }
    }

    
    
    fn function_types(&mut self, f: &'ast ast::Function) -> Result<(Vec<CType>, CType), CodegenError> {
        let params = f
            .params
            .iter()
            .map(|p| self.ctype_from_ast(&p.ty, &p.span))
            .collect::<Result<Vec<_>, _>>()?;
        let return_ty = match &f.return_ty {
            Some(ty) => self.ctype_from_ast(ty, &f.span)?,
            None => CType::Void,
        };
        Ok((params, return_ty))
    }

    
    
    

    
    
    
    fn ctype_from_ast(&self, ty: &ast::Type, span: &SourceSpan) -> Result<CType, CodegenError> {
        match ty {
            ast::Type::Primitive(p) => Ok(CType::from_primitive(*p)),
            ast::Type::Pointer { pointee, mutability, .. } => Ok(CType::Pointer {
                pointee: Box::new(self.ctype_from_ast(pointee, span)?),
                mutable: *mutability == ast::PointerMutability::Mut,
            }),
            ast::Type::Array { element, length, .. } => {
                let elem = self.ctype_from_ast(element, span)?;
                let len = match length.as_ref() {
                    ast::Expr::Literal(ast::LiteralExpr::Integer(n), _) if *n >= 0 => *n as u64,
                    _ => return Err(CodegenError::Internal("array length must be a compile-time integer".to_string())),
                };
                Ok(CType::Array { element: Box::new(elem), length: len })
            }
            ast::Type::Named { name, args, .. } => {
                if name == "Self" {
                    let struct_name = self
                        .current_struct
                        .as_ref()
                        .ok_or_else(|| unsupported(span, "`Self` may only be used inside an impl block"))?;
                    let generics = self
                        .structs
                        .get(struct_name)
                        .map(|i| i.generics.clone())
                        .unwrap_or_default();
                    let resolved = generics
                        .iter()
                        .map(|g| self.current_generics.get(g).cloned().unwrap_or_else(|| CType::Generic(g.clone())))
                        .collect();
                    return Ok(CType::Struct { name: struct_name.clone(), args: resolved });
                }
                if self.type_aliases.contains_key(name) {
                    if !args.is_empty() {
                        return Err(unsupported(span, format!("type alias `{name}` cannot have arguments")));
                    }
                    return Ok(self.type_aliases.get(name).cloned().unwrap());
                }
                if self.current_generics.contains_key(name) {
                    if !args.is_empty() {
                        return Err(unsupported(span, format!("type parameter `{name}` cannot have arguments")));
                    }
                    return Ok(self.current_generics.get(name).cloned().unwrap());
                }
                if !self.structs.contains_key(name) {
                    return Err(CodegenError::Internal(format!("unknown type `{name}` in codegen")));
                }
                let resolved = args
                    .iter()
                    .map(|a| self.ctype_from_ast(a, span))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(CType::Struct { name: name.clone(), args: resolved })
            }
        }
    }

    
    
    fn llvm_type(&mut self, ty: &CType) -> Result<BasicTypeEnum<'ctx>, CodegenError> {
        let result = match ty {
            CType::Void => return Err(CodegenError::Internal("void has no value type".to_string())),
            CType::Bool => self.context.bool_type().as_basic_type_enum(),
            CType::Char | CType::I8 | CType::U8 => self.context.i8_type().as_basic_type_enum(),
            CType::I16 | CType::U16 => self.context.i16_type().as_basic_type_enum(),
            CType::I32 | CType::U32 => self.context.i32_type().as_basic_type_enum(),
            CType::I64 | CType::U64 | CType::Isize | CType::Usize => self.context.i64_type().as_basic_type_enum(),
            CType::I128 | CType::U128 => self.context.i128_type().as_basic_type_enum(),
            CType::F32 => self.context.f32_type().as_basic_type_enum(),
            CType::F64 => self.context.f64_type().as_basic_type_enum(),
            CType::Str => self.context.ptr_type(AddressSpace::default()).as_basic_type_enum(),
            CType::Null => self.context.ptr_type(AddressSpace::default()).as_basic_type_enum(),
            CType::Pointer { .. } => self.context.ptr_type(AddressSpace::default()).as_basic_type_enum(),
            CType::Generic(name) => match self.current_generics.get(name).cloned() {
                Some(concrete) => return self.llvm_type(&concrete),
                None => {
                    return Err(CodegenError::Internal(format!("unresolved generic `{name}` survived to codegen")))
                }
            },
            CType::IntLiteral => return Err(CodegenError::Internal("IntLiteral should have been resolved".to_string())),
            CType::FloatLiteral => return Err(CodegenError::Internal("FloatLiteral should have been resolved".to_string())),
            CType::Struct { name, args } => {
                self.struct_type(name, args)?.as_basic_type_enum()
            }
            CType::Array { element, length } => {
                let elem_ty = self.llvm_type(element)?;
                elem_ty.array_type(*length as u32).as_basic_type_enum()
            }
        };
        Ok(result)
    }

    
    
    
    fn struct_type(&mut self, name: &str, args: &[CType]) -> Result<StructType<'ctx>, CodegenError> {
        let key = struct_key(name, args);
        if let Some(st) = self.struct_types.get(&key) {
            return Ok(*st);
        }
        let info = self
            .structs
            .get(name)
            .ok_or_else(|| CodegenError::Internal(format!("struct `{name}` was never declared")))?;
        let bindings: HashMap<String, CType> = info
            .generics
            .iter()
            .zip(args.iter())
            .map(|(g, ty)| (g.clone(), ty.clone()))
            .collect();
        let fields: Vec<CType> = info
            .fields
            .iter()
            .map(|(_, ty)| ty.substitute(&bindings))
            .collect();
        let llvm_fields = fields
            .iter()
            .map(|ty| self.llvm_type(ty))
            .collect::<Result<Vec<_>, _>>()?;
        let st = self.context.struct_type(&llvm_fields, false);
        self.struct_types.insert(key, st);
        Ok(st)
    }

    
    
    

    fn push_scope(&mut self) {
        self.var_stack.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.var_stack.pop();
    }

    fn define_var(&mut self, name: &str, ptr: PointerValue<'ctx>, ty: CType) {
        self.var_stack.last_mut().expect("scope stack is never empty").insert(name.to_string(), (ptr, ty));
    }

    fn lookup_var(&self, name: &str) -> Option<(PointerValue<'ctx>, CType)> {
        self.var_stack.iter().rev().find_map(|scope| scope.get(name)).cloned()
    }

    
    
    

    
    
    
    fn ensure_not_terminated(&mut self) {
        if self.is_terminated() {
            let dead = self.append_block("dead");
            self.builder.position_at_end(dead);
        }
    }

    fn is_terminated(&self) -> bool {
        self.builder.get_insert_block().is_some_and(|block| block.get_terminator().is_some())
    }

    fn append_block(&mut self, name: &str) -> inkwell::basic_block::BasicBlock<'ctx> {
        self.context.append_basic_block(self.current_fn.expect("a function must be active"), name)
    }

    fn branch_to(&mut self, dest: inkwell::basic_block::BasicBlock<'ctx>) {
        if !self.is_terminated() {
            let _ = self.builder.build_unconditional_branch(dest);
        }
    }

    fn load(&mut self, ptr: PointerValue<'ctx>, ty: &CType) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let pointee_ty = self.llvm_type(ty)?;
        self.builder
            .build_load(pointee_ty, ptr, "load")
            .map_err(|err| CodegenError::Internal(err.to_string()))
    }

    fn int_err(&self, err: inkwell::builder::BuilderError) -> CodegenError {
        CodegenError::Internal(err.to_string())
    }

    
    
    

    fn gen_function(&mut self, task: &PendingFunc<'ast>) -> Result<(), CodegenError> {
        let f = task.f;
        let struct_name = task.struct_name.as_deref();
        let key = self.instance_key(f.name.as_str(), struct_name, &task.struct_args, &task.fn_args);
        let fn_value = self
            .lookup_instance(struct_name, &key)
            .ok_or_else(|| CodegenError::Internal(format!("function instance `{key}` was never declared")))?;

        let prev_struct = self.current_struct.take();
        self.current_struct = struct_name.map(str::to_string);
        let prev_args = std::mem::take(&mut self.current_struct_args);
        let prev_gen = std::mem::take(&mut self.current_generics);
        self.current_struct_args = task.struct_args.clone();
        self.current_generics = self.resolve_generics(f, struct_name, &task.struct_args, &task.fn_args)?;
        let return_ty = match &f.return_ty {
            Some(ty) => self.ctype_from_ast(ty, &f.span)?,
            None => CType::Void,
        };
        self.current_fn = Some(fn_value);
        self.current_fn_return = return_ty.clone();

        let entry = self.context.append_basic_block(fn_value, "entry");
        self.builder.position_at_end(entry);

        self.push_scope();
        for (i, param) in f.params.iter().enumerate() {
            let param_cty = self.ctype_from_ast(&param.ty, &param.span)?;
            let param_ty = self.llvm_type(&param_cty)?;
            let slot = self
                .builder
                .build_alloca(param_ty, &param.name)
                .map_err(|err| self.int_err(err))?;
            let value = fn_value
                .get_nth_param(i as u32)
                .ok_or_else(|| CodegenError::Internal(format!("missing parameter {i} of `{}`", f.name)))?;
            self.builder.build_store(slot, value).map_err(|err| self.int_err(err))?;
            self.define_var(&param.name, slot, param_cty);
        }

        self.gen_block(&f.body)?;

        self.cleanup_scope();
        self.ensure_not_terminated();
        if return_ty == CType::Void {
            self.builder.build_return(None).map_err(|err| self.int_err(err))?;
        } else {
            let zero = self.zero_of(&return_ty)?;
            self.builder.build_return(Some(&zero)).map_err(|err| self.int_err(err))?;
        }

        self.pop_scope();
        self.current_struct = prev_struct;
        self.current_struct_args = prev_args;
        self.current_generics = prev_gen;
        self.current_fn = None;
        self.current_fn_return = CType::Void;
        Ok(())
    }

    fn zero_of(&mut self, ty: &CType) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let basic = self.llvm_type(ty)?;
        Ok(basic.const_zero())
    }

    
    
    

    fn gen_block(&mut self, block: &'ast ast::Block) -> Result<(), CodegenError> {
        self.push_scope();
        for stmt in &block.stmts {
            self.gen_stmt(stmt)?;
            if self.is_terminated() {
                break;
            }
        }
        if !self.is_terminated() {
            self.cleanup_scope();
        }
        self.pop_scope();
        Ok(())
    }

    fn cleanup_scope(&mut self) {
        
        
        
        
        
        
        self.var_stack.last_mut().map(|scope| scope.clear());
    }

    fn gen_stmt(&mut self, stmt: &'ast ast::Stmt) -> Result<(), CodegenError> {
        self.ensure_not_terminated();
        match stmt {
            ast::Stmt::Let(s) => self.gen_let(s),
            ast::Stmt::Expr(e) => {
                self.emit_expr(e).map(|_| ())
            }
            ast::Stmt::If(s) => self.gen_if(s),
            ast::Stmt::While(s) => self.gen_while(s),
            ast::Stmt::For(s) => self.gen_for(s),
            ast::Stmt::Switch(s) => self.gen_switch(s),
            ast::Stmt::Break(s) => self.gen_break(s),
            ast::Stmt::Continue(s) => self.gen_continue(s),
            ast::Stmt::Block(b) => self.gen_block(b),
        }
    }

    fn gen_let(&mut self, s: &'ast ast::LetStmt) -> Result<(), CodegenError> {
        let (init_value, init_ty) = match &s.init {
                Some(init) => {
                    let prev_hint = self.struct_literal_hint.take();
                    if let Some(declared) = &s.ty {
                        let declared_ty = self.ctype_from_ast(declared, &s.span)?;
                        if let CType::Struct { name: dn, args } = &declared_ty
                            && !args.is_empty()
                        {
                            
                            
                            
                            let matches = match init {
                                ast::Expr::StructLiteral { name, .. } => name == dn,
                                ast::Expr::Call { callee, .. } => matches!(
                                    callee.as_ref(),
                                    ast::Expr::Path { segments, .. }
                                        if segments.len() == 2 && &segments[0] == dn
                                ),
                                _ => false,
                            };
                            if matches {
                                self.struct_literal_hint = Some((dn.clone(), args.clone()));
                            }
                        }
                    }
                    let result = self.emit_expr(init);
                    self.struct_literal_hint = prev_hint;
                    result?
                }
            None => {
                let declared = s
                    .ty
                    .as_ref()
                    .ok_or_else(|| unsupported(&s.span, format!("`let {}` requires a type or an initializer", s.name)))?;
                let ty = self.ctype_from_ast(declared, &s.span)?;
                (self.zero_of(&ty)?, ty)
            }
        };
        if init_ty == CType::Void {
            return Err(unsupported(&s.span, format!("`{}` has no value", s.name)));
        }
        
        
        
        let ty = match &s.ty {
            Some(declared) => self.ctype_from_ast(declared, &s.span)?,
            None => default_concrete(&init_ty),
        };
        let slot_ty = self.llvm_type(&ty)?;
        let slot = self
            .builder
            .build_alloca(slot_ty, &s.name)
            .map_err(|err| self.int_err(err))?;
        let value = self.coerce(init_value, &init_ty, &ty)?;
        self.store_into(slot, value, &ty)?;
        self.define_var(&s.name, slot, ty);
        Ok(())
    }

    fn gen_if(&mut self, s: &'ast ast::IfStmt) -> Result<(), CodegenError> {
        let (cond, _) = self.emit_expr(&s.condition)?;
        let cond = cond.into_int_value();

        let then_bb = self.append_block("if.then");
        let else_bb = if s.else_branch.is_some() { Some(self.append_block("if.else")) } else { None };
        let merge_bb = self.append_block("if.end");

        self.builder
            .build_conditional_branch(cond, then_bb, else_bb.unwrap_or(merge_bb))
            .map_err(|err| self.int_err(err))?;

        self.builder.position_at_end(then_bb);
        self.gen_block(&s.then_block)?;
        self.branch_to(merge_bb);

        if let Some(branch) = &s.else_branch {
            self.builder.position_at_end(else_bb.expect("else block exists"));
            match branch {
                ast::ElseBranch::If(inner) => self.gen_if_chain(inner, merge_bb),
                ast::ElseBranch::Block(b) => {
                    self.gen_block(b)?;
                    self.branch_to(merge_bb);
                }
            }
        }

        self.builder.position_at_end(merge_bb);
        Ok(())
    }

    
    
    fn gen_if_chain(&mut self, s: &'ast ast::IfStmt, merge_bb: inkwell::basic_block::BasicBlock<'ctx>) {
        let (cond, _) = match self.emit_expr(&s.condition) {
            Ok(v) => v,
            Err(_) => return,
        };
        let cond = cond.into_int_value();
        let then_bb = self.append_block("if.then");
        let else_bb = if s.else_branch.is_some() { Some(self.append_block("if.else")) } else { None };

        if self.builder.build_conditional_branch(cond, then_bb, else_bb.unwrap_or(merge_bb)).is_err() {
            return;
        }
        self.builder.position_at_end(then_bb);
        let _ = self.gen_block(&s.then_block);
        self.branch_to(merge_bb);

        if let Some(branch) = &s.else_branch {
            self.builder.position_at_end(else_bb.expect("else block exists"));
            match branch {
                ast::ElseBranch::If(inner) => self.gen_if_chain(inner, merge_bb),
                ast::ElseBranch::Block(b) => {
                    let _ = self.gen_block(b);
                    self.branch_to(merge_bb);
                }
            }
        }
    }

    fn gen_while(&mut self, s: &'ast ast::WhileStmt) -> Result<(), CodegenError> {
        let cond_bb = self.append_block("while.cond");
        let body_bb = self.append_block("while.body");
        let exit_bb = self.append_block("while.exit");

        self.builder.build_unconditional_branch(cond_bb).map_err(|err| self.int_err(err))?;

        self.builder.position_at_end(cond_bb);
        let (cond, _) = self.emit_expr(&s.condition)?;
        let cond = cond.into_int_value();
        self.builder.build_conditional_branch(cond, body_bb, exit_bb).map_err(|err| self.int_err(err))?;

        self.builder.position_at_end(body_bb);
        self.control_flow.push(ControlFlowTarget::Loop { continue_bb: cond_bb, exit_bb });
        self.gen_block(&s.body)?;
        self.control_flow.pop();
        if !self.is_terminated() {
            self.builder.build_unconditional_branch(cond_bb).map_err(|err| self.int_err(err))?;
        }

        self.builder.position_at_end(exit_bb);
        Ok(())
    }

    
    
    
    fn gen_for(&mut self, s: &'ast ast::ForStmt) -> Result<(), CodegenError> {
        let (iterable, iter_ty) = self.emit_expr(&s.iterable)?;
        if !matches!(iter_ty, CType::Str)
            && !matches!(&iter_ty, CType::Pointer { pointee, .. } if **pointee == CType::Char)
        {
            return Err(unsupported(
                &s.span,
                "`for` over pointers currently requires a character pointer or string literal",
            ));
        }
        let iter = iterable.into_pointer_value();

        let i64 = self.context.i64_type();
        let idx_slot = self
            .builder
            .build_alloca(i64, "for.idx")
            .map_err(|err| self.int_err(err))?;
        self.builder.build_store(idx_slot, i64.const_int(0, false)).map_err(|err| self.int_err(err))?;

        let cond_bb = self.append_block("for.cond");
        let body_bb = self.append_block("for.body");
        let inc_bb = self.append_block("for.inc");
        let exit_bb = self.append_block("for.exit");

        self.builder.build_unconditional_branch(cond_bb).map_err(|err| self.int_err(err))?;

        
        self.builder.position_at_end(cond_bb);
        let idx = self.builder.build_load(i64, idx_slot, "for.idx").map_err(|err| self.int_err(err))?.into_int_value();
        let char_ty = self.llvm_type(&CType::Char)?;
        let elem_ptr = self.gep(char_ty, iter, idx, "for.elem")?;
        let elem = self
            .builder
            .build_load(self.context.i8_type(), elem_ptr, "for.val")
            .map_err(|err| self.int_err(err))?
            .into_int_value();
        let nonzero = self
            .builder
            .build_int_compare(IntPredicate::NE, elem, self.context.i8_type().const_int(0, false), "for.cond")
            .map_err(|err| self.int_err(err))?;
        self.builder.build_conditional_branch(nonzero, body_bb, exit_bb).map_err(|err| self.int_err(err))?;

        
        
        self.builder.position_at_end(body_bb);
        self.push_scope();
        let var_slot = self
            .builder
            .build_alloca(self.context.i8_type(), &s.variable)
            .map_err(|err| self.int_err(err))?;
        self.builder.build_store(var_slot, elem).map_err(|err| self.int_err(err))?;
        self.define_var(&s.variable, var_slot, CType::Char);
        self.control_flow.push(ControlFlowTarget::Loop { continue_bb: inc_bb, exit_bb });
        self.gen_block(&s.body)?;
        self.control_flow.pop();
        self.pop_scope();
        self.branch_to(inc_bb);

        
        self.builder.position_at_end(inc_bb);
        let next = self
            .builder
            .build_int_add(idx, i64.const_int(1, false), "for.next")
            .map_err(|err| self.int_err(err))?;
        self.builder.build_store(idx_slot, next).map_err(|err| self.int_err(err))?;
        self.builder.build_unconditional_branch(cond_bb).map_err(|err| self.int_err(err))?;

        self.builder.position_at_end(exit_bb);
        Ok(())
    }

    
    
    fn gen_switch(&mut self, s: &'ast ast::SwitchStmt) -> Result<(), CodegenError> {
        let (value, value_ty) = self.emit_expr(&s.value)?;

        let arm_bbs: Vec<inkwell::basic_block::BasicBlock<'ctx>> =
            s.arms.iter().map(|_| self.append_block("switch.arm")).collect();
        let merge_bb = self.append_block("switch.exit");

        let default_idx = s.arms.iter().position(|arm| arm.pattern.is_none());
        let fallback = match default_idx {
            Some(d) => arm_bbs[d],
            None => merge_bb,
        };

        
        for (i, arm) in s.arms.iter().enumerate() {
            let Some(pattern) = &arm.pattern else {
                continue;
            };
            let (pat, pat_ty) = self.emit_expr(pattern)?;
            let unified = CType::unify(&value_ty, &pat_ty).unwrap_or_else(|| value_ty.clone());
            let concrete = default_concrete(&unified);
            if !concrete.is_numeric() {
                return Err(unsupported(
                    &arm.span,
                    format!("`switch` on non-numeric type `{concrete}` is not supported"),
                ));
            }
            let lv = self.coerce(value, &value_ty, &concrete)?;
            let rv = self.coerce(pat, &pat_ty, &concrete)?;
            let cond = if concrete.is_float() {
                self.builder
                    .build_float_compare(FloatPredicate::OEQ, lv.into_float_value(), rv.into_float_value(), "switch.cmp")
                    .map_err(|err| self.int_err(err))?
            } else {
                self.builder
                    .build_int_compare(IntPredicate::EQ, lv.into_int_value(), rv.into_int_value(), "switch.cmp")
                    .map_err(|err| self.int_err(err))?
            };
            let cont_bb = self.append_block("switch.cont");
            self.builder
                .build_conditional_branch(cond, arm_bbs[i], cont_bb)
                .map_err(|err| self.int_err(err))?;
            self.builder.position_at_end(cont_bb);
        }
        
        self.builder.build_unconditional_branch(fallback).map_err(|err| self.int_err(err))?;

        
        for (i, arm) in s.arms.iter().enumerate() {
            self.builder.position_at_end(arm_bbs[i]);
            self.control_flow.push(ControlFlowTarget::Switch { exit_bb: merge_bb });
            for stmt in &arm.body {
                self.gen_stmt(stmt)?;
                if self.is_terminated() {
                    break;
                }
            }
            self.control_flow.pop();
            let next = if i + 1 < s.arms.len() { arm_bbs[i + 1] } else { merge_bb };
            self.branch_to(next);
        }

        self.builder.position_at_end(merge_bb);
        Ok(())
    }

    fn gen_break(&mut self, s: &'ast ast::BreakStmt) -> Result<(), CodegenError> {
        let target = self
            .control_flow
            .last()
            .ok_or_else(|| unsupported(&s.span, "`break` outside a loop or switch"))?;
        let exit_bb = match target {
            ControlFlowTarget::Loop { exit_bb, .. } | ControlFlowTarget::Switch { exit_bb } => *exit_bb,
        };
        self.builder.build_unconditional_branch(exit_bb).map_err(|err| self.int_err(err))?;
        Ok(())
    }

    fn gen_continue(&mut self, s: &'ast ast::ContinueStmt) -> Result<(), CodegenError> {
        let target = self
            .control_flow
            .iter()
            .rev()
            .find(|target| matches!(target, ControlFlowTarget::Loop { .. }))
            .ok_or_else(|| unsupported(&s.span, "`continue` outside a loop"))?;
        let continue_bb = match target {
            ControlFlowTarget::Loop { continue_bb, .. } => *continue_bb,
            ControlFlowTarget::Switch { .. } => unreachable!("filtered above"),
        };
        self.builder.build_unconditional_branch(continue_bb).map_err(|err| self.int_err(err))?;
        Ok(())
    }

    
    fn gep(
        &mut self,
        pointee: BasicTypeEnum<'ctx>,
        ptr: PointerValue<'ctx>,
        index: inkwell::values::IntValue<'ctx>,
        name: &str,
    ) -> Result<PointerValue<'ctx>, CodegenError> {
        unsafe { self.builder.build_gep(pointee, ptr, &[index], name) }.map_err(|err| self.int_err(err))
    }

    
    
    

    
    
    
    fn emit_expr(&mut self, e: &'ast ast::Expr) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        match e {
            ast::Expr::Literal(lit, span) => self.emit_literal(lit, span),
            ast::Expr::Var { name, span } => {
                if let Some((ptr, ty)) = self.lookup_var(name) {
                    return Ok((self.load(ptr, &ty)?, ty));
                }
                if let Some(expr) = self.const_values.get(name) {
                    return self.emit_expr(expr);
                }
                Err(unsupported(span, format!("undefined variable `{name}`")))
            }
            ast::Expr::Call { callee, args, span } => self.emit_call(callee, args, span),
            ast::Expr::MethodCall { receiver, method, args, span } => self.emit_method_call(receiver, method, args, span),
            ast::Expr::FieldAccess { base, field, span } => {
                let (ptr, ty) = self.field_addr(base, field, span)?;
                Ok((self.load(ptr, &ty)?, ty))
            }
            ast::Expr::Index { base, index, span } => {
                let (ptr, ty) = self.index_addr(base, index, span)?;
                Ok((self.load(ptr, &ty)?, ty))
            }
            ast::Expr::Unary { op, operand, span } => self.emit_unary(*op, operand, span),
            ast::Expr::Binary { op, lhs, rhs, span } => self.emit_binary(*op, lhs, rhs, span),
            ast::Expr::Assign { target, op, value, span } => self.emit_assign(target, *op, value, span),
            ast::Expr::Cast { expr: inner, ty, span } => self.emit_cast(inner, &*ty, span),
            ast::Expr::StructLiteral { name, args, fields, span } => self.emit_struct_literal(name, args, fields, span),
            ast::Expr::Match { scrutinee, arms, span } => self.emit_match(scrutinee, arms, span),
            ast::Expr::Return { value, span } => self.emit_return(value.as_deref(), span),
            ast::Expr::Path { segments, span } => {
                if segments.len() == 2 {
                    if let Some(variants) = self.enums.get(&segments[0]) {
                        if let Some((_, Some(value))) = variants.iter().find(|(name, _)| name == &segments[1]) {
                            let value = self.context.i32_type().const_int((*value) as u64, false);
                            return Ok((value.as_basic_value_enum(), CType::I32));
                        }
                        if let Some(index) = variants.iter().position(|(name, _)| name == &segments[1]) {
                            let value = self.context.i32_type().const_int(index as u64, false);
                            return Ok((value.as_basic_value_enum(), CType::I32));
                        }
                    }
                }
                Err(unsupported(span, format!("`{}` is not a value", segments.join("::"))))
            }
            ast::Expr::ArrayRepeat { value, count, span } => {
                self.emit_array_repeat(value, count, span)
            }
            ast::Expr::ArrayLiteral { elements, span } => {
                self.emit_array_literal(elements, span)
            }
            ast::Expr::Range { start, .. } => {
                
                
                let (val, ty) = self.emit_expr(start)?;
                Ok((val, ty))
            }
            ast::Expr::SizeOf { ty, span } => self.emit_sizeof(ty, span),
        }
    }

    fn emit_sizeof(
        &mut self,
        ty: &ast::Type,
        span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        let cty = self.ctype_from_ast(ty, span)?;
        let llvm_ty = self.llvm_type(&cty)?;
        let target_data = TargetData::create(&self.data_layout);
        let size = target_data.get_abi_size(&llvm_ty);
        let value = self.context.i64_type().const_int(size, false);
        Ok((value.as_basic_value_enum(), CType::Usize))
    }

    fn emit_array_repeat(
        &mut self,
        value: &'ast ast::Expr,
        count: &ast::Expr,
        span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        let (val, elem_ty) = self.emit_expr(value)?;
        let llvm_elem = self.llvm_type(&elem_ty)?;
        let len = match count {
            ast::Expr::Literal(ast::LiteralExpr::Integer(n), _) if *n >= 0 => *n as u32,
            _ => return Err(unsupported(span, "array length must be a compile-time integer")),
        };
        let array_ty = llvm_elem.array_type(len);
        let alloca = self.builder.build_alloca(array_ty, "array")
            .map_err(|e| CodegenError::Internal(format!("alloca failed: {e}")))?;
        for i in 0..len {
            let idx = self.context.i32_type().const_int(i as u64, false);
            let ptr = unsafe {
                self.builder.build_gep(llvm_elem, alloca, &[idx], "elem_ptr")
                    .map_err(|e| CodegenError::Internal(format!("gep failed: {e}")))?
            };
            self.builder.build_store(ptr, val)
                .map_err(|e| CodegenError::Internal(format!("store failed: {e}")))?;
        }
        let arr_ty = CType::Array { element: Box::new(elem_ty), length: len as u64 };
        Ok((alloca.as_basic_value_enum(), arr_ty))
    }

    fn emit_array_literal(
        &mut self,
        elements: &'ast [ast::Expr],
        span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        if elements.is_empty() {
            return Err(unsupported(span, "empty array literal needs a type annotation"));
        }
        let (first_val, elem_ty) = self.emit_expr(&elements[0])?;
        let llvm_elem = self.llvm_type(&elem_ty)?;
        let len = elements.len() as u32;
        let array_ty = llvm_elem.array_type(len);
        let alloca = self.builder.build_alloca(array_ty, "array")
            .map_err(|e| CodegenError::Internal(format!("alloca failed: {e}")))?;
        
        let idx0 = self.context.i32_type().const_int(0, false);
        let ptr0 = unsafe {
            self.builder.build_gep(llvm_elem, alloca, &[idx0], "elem_ptr")
                .map_err(|e| CodegenError::Internal(format!("gep failed: {e}")))?
        };
        self.builder.build_store(ptr0, first_val)
            .map_err(|e| CodegenError::Internal(format!("store failed: {e}")))?;
        
        for (i, elem) in elements[1..].iter().enumerate() {
            let (val, _) = self.emit_expr(elem)?;
            let idx = self.context.i32_type().const_int((i + 1) as u64, false);
            let ptr = unsafe {
                self.builder.build_gep(llvm_elem, alloca, &[idx], "elem_ptr")
                    .map_err(|e| CodegenError::Internal(format!("gep failed: {e}")))?
            };
            self.builder.build_store(ptr, val)
                .map_err(|e| CodegenError::Internal(format!("store failed: {e}")))?;
        }
        let arr_ty = CType::Array { element: Box::new(elem_ty), length: len as u64 };
        Ok((alloca.as_basic_value_enum(), arr_ty))
    }

    fn emit_literal(&mut self, lit: &ast::LiteralExpr, span: &SourceSpan) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        let context = self.context;
        let value = match lit {
            ast::LiteralExpr::Integer(v) => (int_literal_value(context, *v).as_basic_value_enum(), CType::IntLiteral),
            ast::LiteralExpr::Float(v) => (context.f64_type().const_float(*v).as_basic_value_enum(), CType::FloatLiteral),
            ast::LiteralExpr::Char(c) => (
                context.i8_type().const_int(u64::from(*c as u8), false).as_basic_value_enum(),
                CType::Char,
            ),
            ast::LiteralExpr::Bool(b) => (
                context.bool_type().const_int(u64::from(*b), false).as_basic_value_enum(),
                CType::Bool,
            ),
            ast::LiteralExpr::String(s) => (self.string_constant(s, span)?.as_basic_value_enum(), CType::Str),
            ast::LiteralExpr::Null => (
                self.context.ptr_type(AddressSpace::default()).const_null().as_basic_value_enum(),
                CType::Null,
            ),
        };
        Ok(value)
    }

    fn emit_unary(
        &mut self,
        op: ast::UnaryOp,
        operand: &'ast ast::Expr,
        span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        let (value, ty) = match op {
            ast::UnaryOp::Deref => {
                let (value, ty) = self.emit_expr(operand)?;
                let pointee = match &ty {
                    CType::Pointer { pointee, .. } => (**pointee).clone(),
                    CType::Str => CType::Char,
                    other => return Err(unsupported(span, format!("cannot dereference a value of type `{other}`"))),
                };
                (self.load(value.into_pointer_value(), &pointee)?, pointee)
            }
            ast::UnaryOp::AddrOf | ast::UnaryOp::AddrOfMut => {
                let (ptr, ty) = self.emit_lvalue(operand, span)?;
                let ptr_ty = CType::Pointer {
                    pointee: Box::new(ty),
                    mutable: op == ast::UnaryOp::AddrOfMut,
                };
                (ptr.as_basic_value_enum(), ptr_ty)
            }
            ast::UnaryOp::Neg => {
                let (value, ty) = self.emit_expr(operand)?;
                let concrete = default_concrete(&ty);
                let value = self.coerce(value, &ty, &concrete)?;
                let neg = if concrete.is_float() {
                    self.builder
                        .build_float_neg(value.into_float_value(), "neg")
                        .map_err(|err| self.int_err(err))?
                        .as_basic_value_enum()
                } else {
                    self.builder
                        .build_int_neg(value.into_int_value(), "neg")
                        .map_err(|err| self.int_err(err))?
                        .as_basic_value_enum()
                };
                (neg, concrete)
            }
            ast::UnaryOp::Not | ast::UnaryOp::BitNot => {
                let (value, _) = self.emit_expr(operand)?;
                let not = self
                    .builder
                    .build_not(value.into_int_value(), "not")
                    .map_err(|err| self.int_err(err))?
                    .as_basic_value_enum();
                let ty = if op == ast::UnaryOp::Not { CType::Bool } else { default_concrete(&self.static_ctype(operand)) };
                (not, ty)
            }
        };
        Ok((value, ty))
    }

    fn binary_op_method_name(op: ast::BinaryOp) -> Option<&'static str> {
        Some(match op {
            ast::BinaryOp::Add => "add",
            ast::BinaryOp::Sub => "sub",
            ast::BinaryOp::Mul => "mul",
            ast::BinaryOp::Div => "div",
            ast::BinaryOp::Mod => "mod",
            ast::BinaryOp::Eq => "eq",
            ast::BinaryOp::NotEq => "ne",
            ast::BinaryOp::Lt => "lt",
            ast::BinaryOp::Gt => "gt",
            ast::BinaryOp::LtEq => "le",
            ast::BinaryOp::GtEq => "ge",
            ast::BinaryOp::AndAnd => "and",
            ast::BinaryOp::OrOr => "or",
            ast::BinaryOp::BitAnd => "bit_and",
            ast::BinaryOp::BitOr => "bit_or",
            ast::BinaryOp::BitXor => "bit_xor",
            ast::BinaryOp::Shl => "shl",
            ast::BinaryOp::Shr => "shr",
        })
    }

    fn emit_binary(
        &mut self,
        op: ast::BinaryOp,
        lhs: &'ast ast::Expr,
        rhs: &'ast ast::Expr,
        span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        use ast::BinaryOp::*;

        if matches!(op, AndAnd | OrOr) {
            return self.emit_logical(op, lhs, rhs);
        }

        if let Some(method_name) = Self::binary_op_method_name(op) {
            let (lv, lt) = self.emit_expr(lhs)?;
            let receiver = match &lt {
                CType::Struct { name, .. } => Some(name.clone()),
                CType::Pointer { pointee, .. } => match &**pointee {
                    CType::Struct { name, .. } => Some(name.clone()),
                    _ => None,
                },
                _ => None,
            };
            if let Some(struct_name) = receiver {
                if self.methods.get(&struct_name).and_then(|m| m.get(method_name)).is_some() {
                    let arg_refs: [&'ast ast::Expr; 1] = [rhs];
                    return self.emit_method_call_ref(lhs, method_name, &arg_refs, span);
                }
            }
            let _ = lv;
        }

        let (lv, lt) = self.emit_expr(lhs)?;
        let (rv, rt) = self.emit_expr(rhs)?;
        let unified = match CType::unify(&lt, &rt) {
            Some(t) => t,
            None => lt.clone(),
        };
        let concrete = default_concrete(&unified);
        let lv = self.coerce(lv, &lt, &concrete)?;
        let rv = self.coerce(rv, &rt, &concrete)?;

        let is_cmp = matches!(op, Eq | NotEq | Lt | Gt | LtEq | GtEq);
        if is_cmp {
            let is_ptr = matches!(concrete, CType::Pointer { .. } | CType::Str | CType::Null);
            let cmp = if concrete.is_float() {
                let pred = float_predicate(op);
                self.builder
                    .build_float_compare(pred, lv.into_float_value(), rv.into_float_value(), "cmp")
                    .map_err(|err| self.int_err(err))?
            } else if is_ptr {
                
                
                let i64 = self.context.i64_type();
                let li = self
                    .builder
                    .build_ptr_to_int(lv.into_pointer_value(), i64, "p2i")
                    .map_err(|err| self.int_err(err))?;
                let ri = self
                    .builder
                    .build_ptr_to_int(rv.into_pointer_value(), i64, "p2i")
                    .map_err(|err| self.int_err(err))?;
                let pred = int_predicate(op, &CType::U64);
                self.builder
                    .build_int_compare(pred, li, ri, "cmp")
                    .map_err(|err| self.int_err(err))?
            } else {
                let pred = int_predicate(op, &concrete);
                self.builder
                    .build_int_compare(pred, lv.into_int_value(), rv.into_int_value(), "cmp")
                    .map_err(|err| self.int_err(err))?
            };
            return Ok((cmp.as_basic_value_enum(), CType::Bool));
        }

        let (value, _) = self.binop_value(op, lv, rv, &concrete, span)?;
        Ok((value, concrete))
    }

    
    fn emit_logical(
        &mut self,
        op: ast::BinaryOp,
        lhs: &'ast ast::Expr,
        rhs: &'ast ast::Expr,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        let (lv, _) = self.emit_expr(lhs)?;
        let lv = lv.into_int_value();

        let bool_ty = self.context.bool_type();
        
        
        
        
        let shortcut = if op == ast::BinaryOp::AndAnd {
            bool_ty.const_int(0, false)
        } else {
            bool_ty.const_int(1, false)
        };

        let rhs_bb = self.append_block("logic.rhs");
        let end_bb = self.append_block("logic.end");
        let cond_block = self.builder.get_insert_block().expect("positioned");

        
        
        
        let (eval, skip) = if op == ast::BinaryOp::AndAnd { (rhs_bb, end_bb) } else { (end_bb, rhs_bb) };
        self.builder
            .build_conditional_branch(lv, eval, skip)
            .map_err(|err| self.int_err(err))?;

        self.builder.position_at_end(rhs_bb);
        let (rv, _) = self.emit_expr(rhs)?;
        let rv = rv.into_int_value();
        let rhs_block = self.builder.get_insert_block().expect("positioned");
        self.builder
            .build_unconditional_branch(end_bb)
            .map_err(|err| self.int_err(err))?;

        self.builder.position_at_end(end_bb);
        let phi = self.builder.build_phi(bool_ty, "logic.res").map_err(|err| self.int_err(err))?;
        phi.add_incoming(&[(&shortcut, cond_block), (&rv, rhs_block)]);
        Ok((phi.as_basic_value(), CType::Bool))
    }

    #[allow(clippy::too_many_arguments)]
    fn binop_value(
        &mut self,
        op: ast::BinaryOp,
        lv: BasicValueEnum<'ctx>,
        rv: BasicValueEnum<'ctx>,
        concrete: &CType,
        span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        if concrete.is_float() {
            let lf = lv.into_float_value();
            let rf = rv.into_float_value();
            let r = match op {
                ast::BinaryOp::Add => self.builder.build_float_add(lf, rf, "add"),
                ast::BinaryOp::Sub => self.builder.build_float_sub(lf, rf, "sub"),
                ast::BinaryOp::Mul => self.builder.build_float_mul(lf, rf, "mul"),
                ast::BinaryOp::Div => self.builder.build_float_div(lf, rf, "div"),
                ast::BinaryOp::Mod => self.builder.build_float_rem(lf, rf, "frem"),
                _ => return Err(unsupported(span, format!("operator `{op:?}` is not valid on floats"))),
            };
            return Ok((r.map_err(|err| self.int_err(err))?.as_basic_value_enum(), concrete.clone()));
        }
        let li = lv.into_int_value();
        let ri = rv.into_int_value();
        let signed = is_signed(concrete);
        let r = match op {
            ast::BinaryOp::Add => self.builder.build_int_add(li, ri, "add"),
            ast::BinaryOp::Sub => self.builder.build_int_sub(li, ri, "sub"),
            ast::BinaryOp::Mul => self.builder.build_int_mul(li, ri, "mul"),
            ast::BinaryOp::Div if signed => self.builder.build_int_signed_div(li, ri, "div"),
            ast::BinaryOp::Div => self.builder.build_int_unsigned_div(li, ri, "udiv"),
            ast::BinaryOp::Mod if signed => self.builder.build_int_signed_rem(li, ri, "rem"),
            ast::BinaryOp::Mod => self.builder.build_int_unsigned_rem(li, ri, "urem"),
            ast::BinaryOp::BitAnd => self.builder.build_and(li, ri, "and"),
            ast::BinaryOp::BitOr => self.builder.build_or(li, ri, "or"),
            ast::BinaryOp::BitXor => self.builder.build_xor(li, ri, "xor"),
            ast::BinaryOp::Shl => self.builder.build_left_shift(li, ri, "shl"),
            ast::BinaryOp::Shr => self.builder.build_right_shift(li, ri, signed, "shr"),
            _ => return Err(unsupported(span, format!("operator `{op:?}` is not valid on integers"))),
        };
        Ok((r.map_err(|err| self.int_err(err))?.as_basic_value_enum(), concrete.clone()))
    }

    

fn maybe_decay_arg(
        &mut self,
        arg: &'ast ast::Expr,
        value: BasicValueEnum<'ctx>,
        vt: &CType,
        expected: Option<&CType>,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        match (expected, vt) {
            (
                Some(CType::Pointer { pointee, mutable }),
                CType::Array { element, .. },
            ) if pointee.unify(element).is_some() => {
                let span = arg.span();
                let ptr = self.emit_lvalue(arg, &span)?.0;
                let derived = CType::Pointer {
                    pointee: pointee.clone(),
                    mutable: *mutable,
                };
                Ok((ptr.as_basic_value_enum(), derived))
            }
            _ => Ok((value, vt.clone())),
        }
    }

fn emit_assign(
        &mut self,
        target: &'ast ast::Expr,
        op: ast::AssignOp,
        value: &'ast ast::Expr,
        span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        let (ptr, ty) = self.emit_lvalue(target, span)?;
        let (vv, vt) = self.emit_expr(value)?;
        let stored = if op == ast::AssignOp::Assign {
            self.coerce(vv, &vt, &ty)?
        } else {
            let current = self.load(ptr, &ty)?;
            let binop = match op {
                ast::AssignOp::Add => ast::BinaryOp::Add,
                ast::AssignOp::Sub => ast::BinaryOp::Sub,
                ast::AssignOp::Mul => ast::BinaryOp::Mul,
                ast::AssignOp::Div => ast::BinaryOp::Div,
                ast::AssignOp::Mod => ast::BinaryOp::Mod,
                ast::AssignOp::Assign => unreachable!(),
            };
            let coerced = self.coerce(vv, &vt, &ty)?;
            let (computed, cty) = self.binop_value(binop, current, coerced, &ty, span)?;
            self.coerce(computed, &cty, &ty)?
        };
        self.store_into(ptr, stored, &ty)?;
        Ok((stored, ty))
    }

    
    
    
    fn store_into(
        &mut self,
        slot: PointerValue<'ctx>,
        value: BasicValueEnum<'ctx>,
        ty: &CType,
    ) -> Result<(), CodegenError> {
        if let CType::Array { element, length } = ty {
            if value.is_pointer_value() {
                let src = value.into_pointer_value();
                let elem_llvm = self.llvm_type(element)?;
                for i in 0..(*length as usize) {
                    let idx = self.context.i32_type().const_int(i as u64, false);
                    let sptr = unsafe {
                        self.builder
                            .build_gep(elem_llvm, src, &[idx], "src")
                            .map_err(|e| CodegenError::Internal(format!("array copy GEP failed: {e}")))?
                    };
                    let dptr = unsafe {
                        self.builder
                            .build_gep(elem_llvm, slot, &[idx], "dst")
                            .map_err(|e| CodegenError::Internal(format!("array copy GEP failed: {e}")))?
                    };
                    let elem = self.load(sptr, element)?;
                    self.builder
                        .build_store(dptr, elem)
                        .map_err(|err| self.int_err(err))?;
                }
                return Ok(());
            }
        }
        self.builder.build_store(slot, value).map_err(|err| self.int_err(err))?;
        Ok(())
    }

    fn emit_cast(
        &mut self,
        inner: &'ast ast::Expr,
        ty: &ast::Type,
        span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        let (value, from) = self.emit_expr(inner)?;
        let to = self.ctype_from_ast(ty, span)?;
        let numeric = (matches!(from, CType::Char | CType::IntLiteral | CType::FloatLiteral)
            || from.is_numeric())
            && (matches!(to, CType::Char | CType::IntLiteral | CType::FloatLiteral)
                || to.is_numeric());
        let pointer = matches!(from, CType::Pointer { .. } | CType::Str)
            && matches!(to, CType::Pointer { .. });
        if !numeric && !pointer {
            return Err(unsupported(span, format!("cannot cast from `{from}` to `{to}`")));
        }
        let value = self.coerce(value, &from, &to)?;
        Ok((value, to))
    }

    fn emit_struct_literal(
        &mut self,
        name: &str,
        args: &'ast [ast::Type],
        fields: &'ast [(String, ast::Expr)],
        span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        let info = self.structs.get(name).cloned().ok_or_else(|| {
            unsupported(span, format!("undefined struct `{name}`"))
        })?;
        let mut emitted: Vec<(Option<CType>, BasicValueEnum<'ctx>, CType)> = Vec::with_capacity(fields.len());
        for (field_name, expr) in fields {
            let ft = info
                .fields
                .iter()
                .find(|(gf, _)| gf == field_name)
                .map(|(_, ty)| ty.clone());
            let (value, vty) = self.emit_expr(expr)?;
            emitted.push((ft, value, vty));
        }
        let mut resolved_args = args
            .iter()
            .map(|a| self.ctype_from_ast(a, span))
            .collect::<Result<Vec<_>, _>>()?;
        if info.generics.is_empty() {
            if !resolved_args.is_empty() {
                return Err(unsupported(span, format!("struct `{name}` has no generic parameters")));
            }
        } else {
            if resolved_args.is_empty() {
                
                
                if let Some((hint_name, hint_args)) = self.struct_literal_hint.take()
                    && hint_name == name
                    && hint_args.len() == info.generics.len()
                {
                    resolved_args = hint_args;
                } else {
                    
                    let mut bindings = HashMap::new();
                    for (ft, _, vt) in &emitted {
                        if let Some(ft) = ft
                            && !match_type(ft, vt, &mut bindings)
                        {
                            return Err(unsupported(span, format!("cannot infer type arguments for `{name}`")));
                        }
                    }
                    resolved_args = info
                        .generics
                        .iter()
                        .map(|g| bindings.get(g).cloned().ok_or_else(|| {
                            unsupported(span, format!("cannot infer type argument `{g}` for `{name}`"))
                        }))
                        .collect::<Result<Vec<_>, _>>()?;
                }
            } else if resolved_args.len() != info.generics.len() {
                return Err(unsupported(
                    span,
                    format!("`{name}` expects {} type argument(s)", info.generics.len()),
                ));
            }
        }
        let struct_ty = CType::Struct {
            name: name.to_string(),
            args: resolved_args.clone(),
        };
        let st = self.struct_type(name, &resolved_args)?;
        let slot = self.builder.build_alloca(st, "struct.lit").map_err(|err| self.int_err(err))?;
        for ((field_name, _), (_, value, vty)) in fields.iter().zip(emitted.iter()) {
            let idx = self
                .field_index(name, &resolved_args, field_name)
                .ok_or_else(|| unsupported(span, format!("`{field_name}` is not a field of `{name}`")))?;
            let expected = self
                .field_type(name, &resolved_args, field_name)
                .ok_or_else(|| CodegenError::Internal(format!("missing field type for `{name}::{field_name}`")))?;
            let value = self.coerce(*value, vty, &expected)?;
            let fptr = self.builder.build_struct_gep(st, slot, idx, "field").map_err(|err| self.int_err(err))?;
            self.builder.build_store(fptr, value).map_err(|err| self.int_err(err))?;
        }
        Ok((self.load(slot, &struct_ty)?, struct_ty))
    }

    fn emit_match(
        &mut self,
        scrutinee: &'ast ast::Expr,
        arms: &'ast [ast::MatchArm],
        _span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        let (scrut, scrut_ty) = self.emit_expr(scrutinee)?;
        let scrut_concrete = default_concrete(&scrut_ty);
        let scrut_int = self.coerce(scrut, &scrut_ty, &scrut_concrete)?.into_int_value();

        let result_ty = arms
            .iter()
            .find_map(|arm| match &arm.body {
                ast::MatchArmBody::Expr(e) => Some(self.static_ctype(e)),
                ast::MatchArmBody::Block(_) => None,
            })
            .unwrap_or(CType::Void);
        let concrete_result = default_concrete(&result_ty);
        let result_slot = if result_ty == CType::Void {
            None
        } else {
            let result_llvm = self.llvm_type(&concrete_result)?;
            Some(self.builder.build_alloca(result_llvm, "match.res").map_err(|err| self.int_err(err))?)
        };

        let first_cmp = self.append_block("match.cmp.0");
        let merge_bb = self.append_block("match.end");
        self.builder.build_unconditional_branch(first_cmp).map_err(|err| self.int_err(err))?;

        
        
        
        
        let cmp_bbs: Vec<_> = (1..arms.len()).map(|_| self.append_block("match.cmp")).collect();
        let body_bbs: Vec<_> = (0..arms.len()).map(|_| self.append_block("match.body")).collect();

        for (i, arm) in arms.iter().enumerate() {
            let cmp_bb = if i == 0 { first_cmp } else { cmp_bbs[i - 1] };
            let body_bb = body_bbs[i];
            let next_bb = if i + 1 < arms.len() { cmp_bbs[i] } else { merge_bb };

            self.builder.position_at_end(cmp_bb);
            match &arm.pattern {
                ast::Pattern::Literal(lit) => {
                    let (pat_val, pat_ty) = self.emit_literal(lit, &arm.span)?;
                    let pat_val = self.coerce(pat_val, &pat_ty, &scrut_concrete)?.into_int_value();
                    let eq = self
                        .builder
                        .build_int_compare(IntPredicate::EQ, scrut_int, pat_val, "match.eq")
                        .map_err(|err| self.int_err(err))?;
                    self.builder.build_conditional_branch(eq, body_bb, next_bb).map_err(|err| self.int_err(err))?;
                }
                ast::Pattern::Binding(_) | ast::Pattern::Wildcard => {
                    self.builder.build_unconditional_branch(body_bb).map_err(|err| self.int_err(err))?;
                }
            }

            self.builder.position_at_end(body_bb);
            self.push_scope();
            if let ast::Pattern::Binding(name) = &arm.pattern {
                let slot_ty = scrut_concrete.clone();
                let bty = self.llvm_type(&slot_ty)?;
                let bslot = self.builder.build_alloca(bty, name).map_err(|err| self.int_err(err))?;
                self.builder.build_store(bslot, scrut).map_err(|err| self.int_err(err))?;
                self.define_var(name, bslot, slot_ty);
            }
            match &arm.body {
                ast::MatchArmBody::Expr(e) => {
                    let (value, vty) = self.emit_expr(e)?;
                    if let Some(slot) = result_slot {
                        let value = self.coerce(value, &vty, &concrete_result)?;
                        self.builder.build_store(slot, value).map_err(|err| self.int_err(err))?;
                    }
                }
                ast::MatchArmBody::Block(b) => {
                    self.gen_block(b)?;
                }
            }
            self.pop_scope();
            self.branch_to(merge_bb);
        }

        self.builder.position_at_end(merge_bb);
        match result_slot {
            Some(slot) => Ok((self.load(slot, &concrete_result)?, concrete_result)),
            None => Ok((self.context.bool_type().const_zero().as_basic_value_enum(), CType::Void)),
        }
    }

    fn emit_return(&mut self, value: Option<&'ast ast::Expr>, span: &SourceSpan) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        let _ = span;
        match value {
            Some(e) => {
                let (v, vt) = self.emit_expr(e)?;
                self.cleanup_scope();
                let return_cty = self
                    .current_fn_return_cty();
                let v = self.coerce(v, &vt, &return_cty)?;
                self.builder.build_return(Some(&v)).map_err(|err| self.int_err(err))?;
                Ok((v, CType::Void))
            }
            None => {
                self.cleanup_scope();
                self.builder.build_return(None).map_err(|err| self.int_err(err))?;
                Ok((self.context.bool_type().const_zero().as_basic_value_enum(), CType::Void))
            }
        }
    }

    
    
    

    fn emit_call(
        &mut self,
        callee: &'ast ast::Expr,
        args: &'ast [ast::Expr],
        span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        match callee {
            ast::Expr::Var { name, .. } => {
                if name == "println" || name == "print" {
                    return self.emit_print(&name.clone(), args, span);
                }
                if self.structs.contains_key(name) {
                    if let Some(methods) = self.methods.get(name)
                        && let Some(sig) = methods.get("new").cloned()
                    {
                        return self.emit_method_associated(name, "new", &sig, &[], args, span);
                    }
                }
                let sig = self
                    .signatures
                    .get(name)
                    .cloned()
                    .ok_or_else(|| unsupported(span, format!("undefined function `{name}`")))?;
                self.emit_user_call(name, &sig, None, Vec::new(), args, span)
            }
            ast::Expr::Path { segments, span: _path_span } => {
                let last = segments.last().cloned().unwrap_or_default();
                if last == "println" || last == "print" {
                    return self.emit_print(&last, args, span);
                }
                if segments.len() == 2 {
                    let struct_name = &segments[0];
                    let method = &segments[1];
                    if struct_name == "Self" {
                        let enclosing = self
                            .current_struct
                            .clone()
                            .ok_or_else(|| unsupported(span, "`Self::` may only be used inside an impl block"))?;
                        let sig = self.methods.get(&enclosing).and_then(|m| m.get(method)).cloned().ok_or_else(|| {
                            CodegenError::Internal(format!("method `Self::{method}` declared but never emitted"))
                        })?;
                        let struct_args = self.current_struct_args.clone();
                        return self.emit_method_associated(&enclosing, method, &sig, &struct_args, args, span);
                    }
                    if let Some(methods) = self.methods.get(struct_name)
                        && let Some(sig) = methods.get(method).cloned()
                    {
                        return self.emit_method_associated(struct_name, method, &sig, &[], args, span);
                    }
                }
                Err(unsupported(span, format!("cannot call `{}`", segments.join("::"))))
            }
            _ => Err(unsupported(span, "expression is not callable")),
        }
    }

    
    
    
    #[allow(clippy::too_many_arguments)]
    fn emit_user_call(
        &mut self,
        name: &str,
        sig_template: &FuncSig,
        struct_name: Option<&str>,
        struct_args: Vec<CType>,
        args: &'ast [ast::Expr],
        span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        let mut values: Vec<BasicValueEnum<'ctx>> = Vec::with_capacity(args.len());
        let mut arg_tys: Vec<CType> = Vec::with_capacity(args.len());
        for arg in args.iter() {
            let (v, vt) = self.emit_expr(arg)?;
            values.push(v);
            arg_tys.push(vt);
        }

        let fn_args = self.infer_fn_args(sig_template, &arg_tys, span)?;
        let key = self.instance_key(name, struct_name, &struct_args, &fn_args);
        let fn_value = self.schedule_instance(key.clone(), name, struct_name, struct_args, fn_args)?;
        let sig = self
            .instance_sigs
            .get(&key)
            .cloned()
            .ok_or_else(|| CodegenError::Internal(format!("instance `{key}` has no resolved signature")))?;
        for (i, _) in args.iter().enumerate() {
            let expected = sig.params.get(i).cloned();
            let (v, vt) = self.maybe_decay_arg(&args[i], values[i], &arg_tys[i], expected.as_ref())?;
            let v = match &expected {
                Some(e) => self.coerce(v, &vt, e)?,
                None => v,
            };
            values[i] = v;
        }
        self.finish_call(fn_value, &sig, values, span)
    }

    
    
    
    #[allow(clippy::too_many_arguments)]
    fn emit_method_associated(
        &mut self,
        struct_name: &str,
        method: &str,
        sig_template: &FuncSig,
        struct_args: &[CType],
        args: &'ast [ast::Expr],
        span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        let mut values: Vec<BasicValueEnum<'ctx>> = Vec::with_capacity(args.len());
        let mut arg_tys: Vec<CType> = Vec::with_capacity(args.len());
        for arg in args.iter() {
            let (v, vt) = self.emit_expr(arg)?;
            values.push(v);
            arg_tys.push(vt);
        }

        let struct_info = self.structs.get(struct_name).cloned().ok_or_else(|| {
            CodegenError::Internal(format!("struct `{struct_name}` was never collected"))
        })?;
        let inferred_struct = if struct_info.generics.is_empty() {
            struct_args.to_vec()
        } else if !struct_args.is_empty() {
            struct_args.to_vec()
        } else {
            
            
            
            
            
            let mut bindings = HashMap::new();
            let params_start = if sig_template.has_self { 1 } else { 0 };
            let bind_ok = sig_template.params[params_start..]
                .iter()
                .zip(arg_tys.iter())
                .all(|(p, a)| match_type(p, a, &mut bindings));
            let fully_bound = struct_info.generics.iter().all(|g| bindings.contains_key(g));
            if bind_ok && fully_bound {
                struct_info
                    .generics
                    .iter()
                    .map(|g| bindings[g].clone())
                    .collect()
            } else if let Some((hint_name, hint_args)) = self.struct_literal_hint.take()
                && hint_name == struct_name
                && hint_args.len() == struct_info.generics.len()
            {
                
                
                hint_args
            } else if bind_ok {
                return Err(unsupported(span, format!("cannot infer all type arguments for `{struct_name}::{method}`")));
            } else {
                return Err(unsupported(span, format!("cannot infer type arguments for `{struct_name}::{method}`")));
            }
        };

        let fn_args = self.infer_fn_args(sig_template, &arg_tys, span)?;
        let key = self.instance_key(method, Some(struct_name), &inferred_struct, &fn_args);
        let fn_value = self.schedule_instance(key.clone(), method, Some(struct_name), inferred_struct.clone(), fn_args)?;
        let sig = self
            .instance_sigs
            .get(&key)
            .cloned()
            .ok_or_else(|| CodegenError::Internal(format!("instance `{key}` has no resolved signature")))?;
        for (i, _) in args.iter().enumerate() {
            let expected = sig.params.get(i).cloned();
            let (v, vt) = self.maybe_decay_arg(&args[i], values[i], &arg_tys[i], expected.as_ref())?;
            let v = match &expected {
                Some(e) => self.coerce(v, &vt, e)?,
                None => v,
            };
            values[i] = v;
        }
        self.finish_call(fn_value, &sig, values, span)
    }

    
    
    
    fn infer_fn_args(
        &self,
        sig_template: &FuncSig,
        arg_tys: &[CType],
        span: &SourceSpan,
    ) -> Result<Vec<CType>, CodegenError> {
        if sig_template.generics.is_empty() {
            return Ok(Vec::new());
        }
        let mut bindings = HashMap::new();
        
        
        
        let params_start = if sig_template.has_self { 1 } else { 0 };
        let ok = sig_template.params[params_start..]
            .iter()
            .zip(arg_tys.iter())
            .all(|(p, a)| match_type(p, a, &mut bindings));
        if !ok {
            return Err(unsupported(span, format!("cannot infer type arguments for `{}`", sig_template.name)));
        }
        let args = sig_template
            .generics
            .iter()
            .map(|g| bindings.get(g).cloned().ok_or_else(|| {
                CodegenError::Unsupported {
                    span: Some(span.clone()),
                    message: format!("cannot infer type argument `{g}` for `{}`", sig_template.name),
                }
            }))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(args)
    }

    
    fn schedule_instance(
        &mut self,
        key: String,
        name: &str,
        struct_name: Option<&str>,
        struct_args: Vec<CType>,
        fn_args: Vec<CType>,
    ) -> Result<FunctionValue<'ctx>, CodegenError> {
        if let Some(v) = self.lookup_instance(struct_name, &key) {
            return Ok(v);
        }
        let f = match struct_name {
            Some(s) => self
                .method_ast
                .get(s)
                .and_then(|m| m.get(name))
                .copied()
                .ok_or_else(|| CodegenError::Internal(format!("method `{s}::{name}` has no AST")))?,
            None => self
                .free_ast
                .get(name)
                .copied()
                .ok_or_else(|| CodegenError::Internal(format!("function `{name}` has no AST")))?,
        };
        self.schedule_instance_inner(key, f, struct_name, struct_args, fn_args)
    }

    
    
    fn schedule_instance_inner(
        &mut self,
        key: String,
        f: &'ast ast::Function,
        struct_name: Option<&str>,
        struct_args: Vec<CType>,
        fn_args: Vec<CType>,
    ) -> Result<FunctionValue<'ctx>, CodegenError> {
        let prev_struct = self.current_struct.take();
        self.current_struct = struct_name.map(str::to_string);
        let prev_args = std::mem::take(&mut self.current_struct_args);
        let prev_gen = std::mem::take(&mut self.current_generics);
        self.current_struct_args = struct_args.clone();
        self.current_generics = self.resolve_generics(f, struct_name, &struct_args, &fn_args)?;
        let (param_ctys, return_cty) = self.function_types(f)?;
        self.current_struct = prev_struct;
        self.current_struct_args = prev_args;
        self.current_generics = prev_gen;

        let params = param_ctys
            .iter()
            .map(|ty| self.llvm_type(ty).map(BasicMetadataTypeEnum::from))
            .collect::<Result<Vec<_>, _>>()?;
        let fn_ty = if return_cty == CType::Void {
            self.context.void_type().fn_type(&params, false)
        } else {
            let return_llvm = self.llvm_type(&return_cty)?;
            return_llvm.fn_type(&params, false)
        };
        let fn_value = self.module.add_function(&key, fn_ty, None);
        fn_value.set_call_conventions(0);

        let has_self = matches!(f.params.first(), Some(p) if p.name == "self");
        let sig = FuncSig {
            name: f.name.clone(),
            generics: f.generics.clone(),
            params: param_ctys,
            has_self,
            is_const: f.is_const,
            is_virtual: f.is_virtual || f.is_abstract,
            is_override: f.is_override,
            is_final: f.is_final,
            is_abstract: f.is_abstract,
            is_static: f.is_static,
            return_ty: if return_cty == CType::Void { None } else { Some(return_cty) },
            struct_name: struct_name.map(str::to_string),
            is_builtin: false,
            is_extern: false,
            is_variadic: false,
        };
        self.instance_sigs.insert(key.clone(), sig);
        match struct_name {
            Some(s) => {
                self.method_funcs.entry(s.to_string()).or_default().insert(key, fn_value);
            }
            None => {
                self.free_funcs.insert(key, fn_value);
            }
        }
        self.pending.push(PendingFunc {
            f,
            struct_name: struct_name.map(str::to_string),
            struct_args,
            fn_args,
        });
        Ok(fn_value)
    }

    fn emit_method_call_ref(
        &mut self,
        receiver: &'ast ast::Expr,
        method: &str,
        args: &[&'ast ast::Expr],
        span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        
        
        let stat = self.static_ctype(receiver);
        let (struct_name, struct_args) = match &stat {
            CType::Struct { name, args } => (name.clone(), args.clone()),
            CType::Pointer { pointee, .. } => match &**pointee {
                CType::Struct { name, args } => (name.clone(), args.clone()),
                other => return Err(unsupported(span, format!("no method `{method}` on `{other}`"))),
            },
            other => return Err(unsupported(span, format!("no method `{method}` on `{other}`"))),
        };
        let sig = self
            .methods
            .get(&struct_name)
            .and_then(|m| m.get(method))
            .cloned()
            .ok_or_else(|| unsupported(span, format!("no method `{method}` on `{struct_name}`")))?;

        let mut values: Vec<BasicValueEnum<'ctx>> = Vec::new();
        if sig.has_self {
            
            
            
            let by_ref = matches!(sig.params.first(), Some(CType::Pointer { .. }));
            if by_ref {
                if let CType::Pointer { .. } = &stat {
                    
                    
                    let (recv, _) = self.emit_expr(receiver)?;
                    values.push(recv);
                } else {
                    match self.emit_lvalue(receiver, span) {
                        Ok((ptr, _)) => values.push(ptr.as_basic_value_enum()),
                        Err(_) => {
                            
                            
                            let (v, vt) = self.emit_expr(receiver)?;
                            let lty = self.llvm_type(&vt)?;
                            let slot = self
                                .builder
                                .build_alloca(lty, "recv_tmp")
                                .map_err(|err| self.int_err(err))?;
                            self.builder.build_store(slot, v).map_err(|err| self.int_err(err))?;
                            values.push(slot.as_basic_value_enum());
                        }
                    }
                }
            } else {
                let (recv, recv_ty) = self.emit_expr(receiver)?;
                match &recv_ty {
                    CType::Pointer { .. } => values.push(self.load(recv.into_pointer_value(), &recv_ty)?),
                    _ => values.push(recv),
                }
            }
        } else {
            return Err(unsupported(span, format!("`{method}` is an associated function and needs no receiver")));
        }

        let mut arg_tys: Vec<CType> = Vec::with_capacity(args.len());
        for arg in args {
            let (v, vt) = self.emit_expr(arg)?;
            values.push(v);
            arg_tys.push(vt);
        }

        let fn_args = self.infer_fn_args(&sig, &arg_tys, span)?;
        let key = self.instance_key(method, Some(&struct_name), &struct_args, &fn_args);
        let fn_value = self.schedule_instance(key.clone(), method, Some(&struct_name), struct_args.clone(), fn_args)?;
        let sig = self
            .instance_sigs
            .get(&key)
            .cloned()
            .ok_or_else(|| CodegenError::Internal(format!("instance `{key}` has no resolved signature")))?;
        for (i, _) in args.iter().enumerate() {
            let expected = sig.params.get(i + 1).cloned();
            let (v, vt) = self.maybe_decay_arg(args[i], values[i + 1], &arg_tys[i], expected.as_ref())?;
            let v = match &expected {
                Some(e) => self.coerce(v, &vt, e)?,
                None => v,
            };
            values[i + 1] = v;
        }
        self.finish_call(fn_value, &sig, values, span)
    }

    fn emit_method_call(
        &mut self,
        receiver: &'ast ast::Expr,
        method: &str,
        args: &'ast [ast::Expr],
        span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        let refs: Vec<&'ast ast::Expr> = args.iter().collect();
        self.emit_method_call_ref(receiver, method, &refs, span)
    }

    fn finish_call(
        &mut self,
        fn_value: FunctionValue<'ctx>,
        sig: &FuncSig,
        values: Vec<BasicValueEnum<'ctx>>,
        span: &SourceSpan,
    ) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        let metadata: Vec<BasicMetadataValueEnum<'ctx>> = values.iter().map(|v| (*v).into()).collect();
        let call = self
            .builder
            .build_call(fn_value, &metadata, "call")
            .map_err(|err| CodegenError::Internal(err.to_string()))?;
        let result_ty = sig.return_ty.clone().unwrap_or(CType::Void);
        let value = if result_ty == CType::Void {
            self.context.bool_type().const_zero().as_basic_value_enum()
        } else {
            call.try_as_basic_value().basic().unwrap_or_else(|| {
                self.context.bool_type().const_zero().as_basic_value_enum()
            })
        };
        let _ = span;
        Ok((value, result_ty))
    }

    fn emit_print(&mut self, name: &str, args: &'ast [ast::Expr], span: &SourceSpan) -> Result<(BasicValueEnum<'ctx>, CType), CodegenError> {
        if args.is_empty() {
            return Err(unsupported(span, "`println`/`print` require at least one argument"));
        }
        let printf = self.print_fn()?;
        let mut fmt = String::new();
        let mut call_args: Vec<BasicMetadataValueEnum<'ctx>> = Vec::new();
        for arg in args {
            let (value, ty) = self.emit_expr(arg)?;
            let (label, promoted) = self.printf_mapping(&value, &ty, span)?;
            fmt.push_str(label);
            match promoted {
                Some(v) => call_args.push(v.into()),
                None => call_args.push(value.into()),
            }
        }
        if name == "println" {
            fmt.push('\n');
        }
        let fmt_ptr = self.string_constant(&fmt, span)?;
        let mut all: Vec<BasicMetadataValueEnum<'ctx>> = vec![fmt_ptr.into()];
        all.extend(call_args);
        self.builder.build_call(printf, &all, "printf").map_err(|err| self.int_err(err))?;
        Ok((self.context.bool_type().const_zero().as_basic_value_enum(), CType::Void))
    }

    
    
    
    fn printf_mapping(
        &mut self,
        value: &BasicValueEnum<'ctx>,
        ty: &CType,
        span: &SourceSpan,
    ) -> Result<(&'static str, Option<BasicValueEnum<'ctx>>), CodegenError> {
        let i32 = self.context.i32_type();
        let f64 = self.context.f64_type();
        Ok(match ty {
            CType::Bool => (
                "%d",
                Some(self.builder.build_int_cast_sign_flag(value.into_int_value(), i32, false, "b").map_err(|err| self.int_err(err))?.as_basic_value_enum()),
            ),
            CType::Char => (
                "%c",
                Some(self.builder.build_int_cast_sign_flag(value.into_int_value(), i32, false, "c").map_err(|err| self.int_err(err))?.as_basic_value_enum()),
            ),
            CType::I8 | CType::I16 => (
                "%d",
                Some(self.builder.build_int_cast_sign_flag(value.into_int_value(), i32, true, "i").map_err(|err| self.int_err(err))?.as_basic_value_enum()),
            ),
            CType::U8 | CType::U16 => (
                "%d",
                Some(self.builder.build_int_cast_sign_flag(value.into_int_value(), i32, false, "u").map_err(|err| self.int_err(err))?.as_basic_value_enum()),
            ),
            CType::I32 | CType::Isize => ("%d", None),
            CType::U32 | CType::Usize => ("%u", None),
            CType::I64 => ("%lld", None),
            CType::U64 => ("%llu", None),
            CType::F32 => (
                "%f",
                Some(self.builder.build_float_cast(value.into_float_value(), f64, "fp").map_err(|err| self.int_err(err))?.as_basic_value_enum()),
            ),
            CType::F64 => ("%f", None),
            CType::FloatLiteral => ("%f", None),
            CType::IntLiteral => ("%lld", None),
            CType::Str => ("%s", None),
            CType::Pointer { pointee, .. } if **pointee == CType::Char => ("%s", None),
            CType::Pointer { .. } => ("%p", None),
            CType::I128 | CType::U128 => {
                let is_signed = matches!(ty, CType::I128);
                let sptr = self.emit_int128_to_string(*value, is_signed, span)?;
                ("%s", Some(sptr))
            }
            other => return Err(unsupported(span, format!("cannot print a value of type `{other}`"))),
        })
    }

        
    
    fn emit_int128_to_string(
        &mut self,
        value: BasicValueEnum<'ctx>,
        is_signed: bool,
        _span: &SourceSpan,
    ) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        let i8t = self.context.i8_type();
        let i64t = self.context.i64_type();
        let i128t = value.get_type().into_int_type();
        let zero128 = i128t.const_zero();
        let ten128 = i128t.const_int(10, false);

        let buf = self.builder.build_alloca(i8t.array_type(48), "buf128").map_err(|err| self.int_err(err))?;
        let pos = self.builder.build_alloca(i64t, "pos").map_err(|err| self.int_err(err))?;
        self.builder.build_store(pos, i64t.const_int(47, false)).map_err(|err| self.int_err(err))?;

        let neg = self.builder.build_alloca(self.context.bool_type(), "neg").map_err(|err| self.int_err(err))?;
        self.builder.build_store(neg, self.context.bool_type().const_zero()).map_err(|err| self.int_err(err))?;

        let mut mag = value.into_int_value();
        if is_signed {
            let is_neg = self.builder.build_int_compare(IntPredicate::SLT, mag, zero128, "isneg").map_err(|err| self.int_err(err))?;
            self.builder.build_store(neg, is_neg).map_err(|err| self.int_err(err))?;
            let negated = self.builder.build_int_sub(zero128, mag, "mag").map_err(|err| self.int_err(err))?;
            mag = self
                .builder
                .build_select(is_neg, negated, mag, "mag")
                .map_err(|err| self.int_err(err))?
                .into_int_value();
        }
        let mag_alloca = self.builder.build_alloca(i128t, "mag").map_err(|err| self.int_err(err))?;
        self.builder.build_store(mag_alloca, mag).map_err(|err| self.int_err(err))?;

        let loop_block = self.context.append_basic_block(self.current_fn.unwrap(), "dloop");
        let done_block = self.context.append_basic_block(self.current_fn.unwrap(), "done");

        
        let cond = self.builder.build_int_compare(IntPredicate::EQ, mag, zero128, "iszero").map_err(|err| self.int_err(err))?;
        let write_zero = self.context.append_basic_block(self.current_fn.unwrap(), "zero");
        self.builder.build_conditional_branch(cond, write_zero, loop_block).map_err(|err| self.int_err(err))?;

        self.builder.position_at_end(write_zero);
        let p = self
            .builder
            .build_load(i64t, pos, "p")
            .map_err(|err| self.int_err(err))?
            .into_int_value();
        
        let char_ptr = unsafe { self.builder.build_gep(i8t.array_type(48), buf, &[i64t.const_zero(), p], "ptr") }.map_err(|err| self.int_err(err))?;
        self.builder.build_store(char_ptr, i8t.const_int('0' as u64, false)).map_err(|err| self.int_err(err))?;
        let sub1 = self.builder.build_int_sub(p, i64t.const_int(1, false), "p1").map_err(|err| self.int_err(err))?;
        self.builder.build_store(pos, sub1).map_err(|err| self.int_err(err))?;
        self.builder.build_unconditional_branch(done_block).map_err(|err| self.int_err(err))?;

        self.builder.position_at_end(loop_block);
        let m = self
            .builder
            .build_load(i128t, mag_alloca, "m")
            .map_err(|err| self.int_err(err))?
            .into_int_value();
        let rem = self.builder.build_int_unsigned_rem(m, ten128, "rem").map_err(|err| self.int_err(err))?;
        let quot = self.builder.build_int_unsigned_div(m, ten128, "quot").map_err(|err| self.int_err(err))?;
        self.builder.build_store(mag_alloca, quot).map_err(|err| self.int_err(err))?;

        let digit = self.builder.build_int_cast(rem, i8t, "digit").map_err(|err| self.int_err(err))?;
        let char_code = self.builder.build_int_add(digit, i8t.const_int('0' as u64, false), "char").map_err(|err| self.int_err(err))?;

        let p = self
            .builder
            .build_load(i64t, pos, "p")
            .map_err(|err| self.int_err(err))?
            .into_int_value();
        let char_ptr = unsafe { self.builder.build_gep(i8t.array_type(48), buf, &[i64t.const_zero(), p], "ptr") }.map_err(|err| self.int_err(err))?;
        self.builder.build_store(char_ptr, char_code).map_err(|err| self.int_err(err))?;
        let p1 = self.builder.build_int_sub(p, i64t.const_int(1, false), "p1").map_err(|err| self.int_err(err))?;
        self.builder.build_store(pos, p1).map_err(|err| self.int_err(err))?;

        let cond = self.builder.build_int_compare(IntPredicate::NE, quot, zero128, "cond").map_err(|err| self.int_err(err))?;
        self.builder.build_conditional_branch(cond, loop_block, done_block).map_err(|err| self.int_err(err))?;

        self.builder.position_at_end(done_block);
        
        let neg_block = self.context.append_basic_block(self.current_fn.unwrap(), "neg");
        let end_block = self.context.append_basic_block(self.current_fn.unwrap(), "end");
        let is_neg = self
            .builder
            .build_load(self.context.bool_type(), neg, "isneg")
            .map_err(|err| self.int_err(err))?
            .into_int_value();
        self.builder.build_conditional_branch(is_neg, neg_block, end_block).map_err(|err| self.int_err(err))?;

        self.builder.position_at_end(neg_block);
        let p = self
            .builder
            .build_load(i64t, pos, "p")
            .map_err(|err| self.int_err(err))?
            .into_int_value();
        let char_ptr = unsafe { self.builder.build_gep(i8t.array_type(48), buf, &[i64t.const_zero(), p], "ptr") }.map_err(|err| self.int_err(err))?;
        self.builder.build_store(char_ptr, i8t.const_int('-' as u64, false)).map_err(|err| self.int_err(err))?;
        let p1 = self.builder.build_int_sub(p, i64t.const_int(1, false), "p1").map_err(|err| self.int_err(err))?;
        self.builder.build_store(pos, p1).map_err(|err| self.int_err(err))?;
        self.builder.build_unconditional_branch(end_block).map_err(|err| self.int_err(err))?;

        self.builder.position_at_end(end_block);
        let final_p = self
            .builder
            .build_load(i64t, pos, "p")
            .map_err(|err| self.int_err(err))?
            .into_int_value();
        
        
        let nul_ptr = unsafe { self.builder.build_gep(i8t.array_type(48), buf, &[i64t.const_zero(), i64t.const_int(47, false)], "nul") }.map_err(|err| self.int_err(err))?;
        self.builder.build_store(nul_ptr, i8t.const_zero()).map_err(|err| self.int_err(err))?;

        let start_idx = self.builder.build_int_add(final_p, i64t.const_int(1, false), "start").map_err(|err| self.int_err(err))?;
        let start_ptr = unsafe { self.builder.build_gep(i8t.array_type(48), buf, &[i64t.const_zero(), start_idx], "str") }.map_err(|err| self.int_err(err))?;
        
        Ok(start_ptr.as_basic_value_enum())
    }

    
    fn emit_lvalue(&mut self, e: &'ast ast::Expr, span: &SourceSpan) -> Result<(PointerValue<'ctx>, CType), CodegenError> {
        match e {
            ast::Expr::Var { name, .. } => {
                let (ptr, ty) = self.lookup_var(name).ok_or_else(|| unsupported(span, format!("undefined variable `{name}`")))?;
                Ok((ptr, ty))
            }
            ast::Expr::FieldAccess { base, field, .. } => self.field_addr(base, field, span),
            ast::Expr::Index { base, index, .. } => self.index_addr(base, index, span),
            ast::Expr::Unary { op: ast::UnaryOp::Deref, operand, .. } => {
                let (value, ty) = self.emit_expr(operand)?;
                match &ty {
                    CType::Pointer { pointee, .. } => Ok((value.into_pointer_value(), (**pointee).clone())),
                    CType::Str => Ok((value.into_pointer_value(), CType::Char)),
                    other => Err(unsupported(span, format!("cannot dereference a value of type `{other}`"))),
                }
            }
            other => Err(unsupported(span, format!("cannot compute the address of `{other:?}`"))),
        }
    }

    fn field_addr(
        &mut self,
        base: &'ast ast::Expr,
        field: &str,
        span: &SourceSpan,
    ) -> Result<(PointerValue<'ctx>, CType), CodegenError> {
        let base_ty = self.static_ctype(base);
        let (struct_name, struct_args, base_ptr) = match base_ty {
            CType::Pointer { pointee, .. } => {
                let st = match *pointee {
                    CType::Struct { name, args } => (name, args),
                    other => {
                        return Err(unsupported(span, format!("field access on a non-struct type `{other}`")));
                    }
                };
                let (bv, _) = self.emit_expr(base)?;
                (st.0, st.1, bv.into_pointer_value())
            }
            CType::Struct { name, args } => {
                if let Ok((bp, _)) = self.emit_lvalue(base, span) {
                    (name, args, bp)
                } else {
                    let (bv, _) = self.emit_expr(base)?;
                    let st = self.struct_type(&name, &args)?;
                    let tmp = self.builder.build_alloca(st, "tmp.struct").map_err(|err| self.int_err(err))?;
                    self.builder.build_store(tmp, bv).map_err(|err| self.int_err(err))?;
                    (name, args, tmp)
                }
            }
            CType::Void => {
                let (bv, brow) = self.emit_expr(base)?;
                match &brow {
                    CType::Pointer { pointee, .. } => {
                        let st = match &**pointee {
                            CType::Struct { name, args } => (name.clone(), args.clone()),
                            other => {
                                return Err(unsupported(span, format!("field access on a non-struct type `{other}`")));
                            }
                        };
                        (st.0, st.1, bv.into_pointer_value())
                    }
                    CType::Struct { name, args } => {
                        let st = self.struct_type(name, args)?;
                        let tmp = self.builder.build_alloca(st, "tmp.struct").map_err(|err| self.int_err(err))?;
                        self.builder.build_store(tmp, bv).map_err(|err| self.int_err(err))?;
                        (name.clone(), args.clone(), tmp)
                    }
                    other => {
                        return Err(unsupported(span, format!("field access on a non-struct type `{other}`")));
                    }
                }
            }
            other => {
                return Err(unsupported(span, format!("field access on a non-struct type `{other}`")));
            }
        };
        let idx = self
            .field_index(&struct_name, &struct_args, field)
            .ok_or_else(|| unsupported(span, format!("`{field}` is not a field of `{struct_name}`")))?;
        let fty = self
            .field_type(&struct_name, &struct_args, field)
            .ok_or_else(|| CodegenError::Internal(format!("missing field type for `{struct_name}::{field}`")))?;
        let st = self.struct_type(&struct_name, &struct_args)?;
        let fptr = self.builder.build_struct_gep(st, base_ptr, idx, "field").map_err(|err| self.int_err(err))?;
        Ok((fptr, fty))
    }

    fn index_addr(
        &mut self,
        base: &'ast ast::Expr,
        index: &'ast ast::Expr,
        span: &SourceSpan,
    ) -> Result<(PointerValue<'ctx>, CType), CodegenError> {
        let (bval, bty) = self.emit_expr(base)?;
        let (pointee, base_ptr) = match &bty {
            CType::Pointer { pointee, .. } => ((**pointee).clone(), bval.into_pointer_value()),
            CType::Str => (CType::Char, bval.into_pointer_value()),
            CType::Array { element, .. } => {
                let arr_ptr = match bval {
                    BasicValueEnum::PointerValue(p) => p,
                    _ => {
                        
                        
                        self.emit_lvalue(base, span)?.0
                    }
                };
                let (ival, ity) = self.emit_expr(index)?;
                let idx = self.coerce(ival, &ity, &CType::I64)?.into_int_value();
                let llvm_arr = self.llvm_type(&bty)?;
                let elem_ptr = unsafe {
                    self.builder
                        .build_gep(
                            llvm_arr,
                            arr_ptr,
                            &[
                                self.context.i64_type().const_int(0, false),
                                idx,
                            ],
                            "elem",
                        )
                        .map_err(|e| CodegenError::Internal(format!("array GEP failed: {e}")))?
                };
                return Ok((elem_ptr, (**element).clone()));
            }
            other => return Err(unsupported(span, format!("value of type `{other}` is not indexable"))),
        };
        let (ival, ity) = self.emit_expr(index)?;
        let idx = self.coerce(ival, &ity, &CType::I64)?.into_int_value();
        let pointee_types = self.llvm_type(&pointee)?;
        let elem_ptr = self.gep(pointee_types, base_ptr, idx, "elem")?;
        Ok((elem_ptr, pointee))
    }

    
    
    

    fn field_index(&self, struct_name: &str, _args: &[CType], field: &str) -> Option<u32> {
        self.structs.get(struct_name)?.fields.iter().position(|(f, _)| f == field).map(|i| i as u32)
    }

    
    
    fn field_type(&self, struct_name: &str, args: &[CType], field: &str) -> Option<CType> {
        let template = self.structs.get(struct_name)?.fields.iter().find(|(f, _)| f == field).map(|(_, ty)| ty.clone())?;
        let bindings: HashMap<String, CType> = self
            .structs
            .get(struct_name)?
            .generics
            .iter()
            .zip(args.iter())
            .map(|(g, ty)| (g.clone(), ty.clone()))
            .collect();
        Some(template.substitute(&bindings))
    }

    
    
    

    
    fn string_constant(&mut self, s: &str, span: &SourceSpan) -> Result<PointerValue<'ctx>, CodegenError> {
        if let Some(ptr) = self.string_globals.get(s) {
            return Ok(*ptr);
        }
        let i8 = self.context.i8_type();
        let mut bytes: Vec<u8> = s.bytes().collect();
        bytes.push(0);
        let arr = i8.array_type(bytes.len() as u32);
        let init: Vec<IntValue<'ctx>> = bytes
            .iter()
            .map(|&b| i8.const_int(u64::from(b), false))
            .collect();
        let arr_init = i8.const_array(&init);
        let name = format!(".str.{}", self.string_counter);
        self.string_counter += 1;
        let global = self.module.add_global(arr, None, &name);
        global.set_initializer(&arr_init);
        global.set_unnamed_addr(true);
        let ptr = global.as_pointer_value();
        let _ = span;
        self.string_globals.insert(s.to_string(), ptr);
        Ok(ptr)
    }

    
    
    
    fn print_fn(&mut self) -> Result<FunctionValue<'ctx>, CodegenError> {
        if let Some(f) = self.printf_fn {
            return Ok(f);
        }
        if let Some(existing) = self.module.get_function("printf") {
            self.printf_fn = Some(existing);
            return Ok(existing);
        }
        let i32 = self.context.i32_type();
        let ptr = self.context.ptr_type(AddressSpace::default());
        let fty = i32.fn_type(&[ptr.into()], true);
        let f = self.module.add_function("printf", fty, None);
        self.printf_fn = Some(f);
        Ok(f)
    }

    
    
    fn coerce(&mut self, value: BasicValueEnum<'ctx>, from: &CType, to: &CType) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        if from == to {
            return Ok(value);
        }
        
        
        if from == &CType::Null && matches!(to, CType::Pointer { .. } | CType::Str) {
            return Ok(value);
        }
        let to_llvm = self.llvm_type(to)?;
        let from_float = matches!(from, CType::F32 | CType::F64 | CType::FloatLiteral);
        let from_int = matches!(from, CType::Bool | CType::Char | CType::IntLiteral) || from.is_int();
        let from_ptr = matches!(from, CType::Pointer { .. } | CType::Str);
        let to_float = matches!(to, CType::F32 | CType::F64);
        let to_int = !to_float && !matches!(to, CType::Pointer { .. } | CType::Str | CType::Struct { .. });

        if from_float && to_float {
            let cast = self
                .builder
                .build_float_cast(value.into_float_value(), to_llvm.into_float_type(), "fcast")
                .map_err(|err| self.int_err(err))?;
            return Ok(cast.as_basic_value_enum());
        }
        if from_float && to_int {
            let target_int = to_llvm.into_int_type();
            let cast = if is_signed(to) {
                self.builder
                    .build_float_to_signed_int(value.into_float_value(), target_int, "f2i")
                    .map_err(|err| self.int_err(err))?
            } else {
                self.builder
                    .build_float_to_unsigned_int(value.into_float_value(), target_int, "f2u")
                    .map_err(|err| self.int_err(err))?
            };
            return Ok(cast.as_basic_value_enum());
        }
        if from_int && to_float {
            let cast = if is_signed(from) || matches!(from, CType::IntLiteral | CType::Bool | CType::Char) {
                self.builder
                    .build_signed_int_to_float(value.into_int_value(), to_llvm.into_float_type(), "i2f")
                    .map_err(|err| self.int_err(err))?
            } else {
                self.builder
                    .build_unsigned_int_to_float(value.into_int_value(), to_llvm.into_float_type(), "i2f")
                    .map_err(|err| self.int_err(err))?
            };
            return Ok(cast.as_basic_value_enum());
        }
        if from_int && to_int {
            let target_int = to_llvm.into_int_type();
            let int = value.into_int_value();
            if int.get_type().get_bit_width() == target_int.get_bit_width() {
                return Ok(value);
            }
            let signed_source = is_signed(from) || matches!(from, CType::IntLiteral | CType::Bool | CType::Char);
            let cast = self
                .builder
                .build_int_cast_sign_flag(int, target_int, signed_source, "icast")
                .map_err(|err| self.int_err(err))?;
            return Ok(cast.as_basic_value_enum());
        }
        if from_ptr && matches!(to, CType::Pointer { .. } | CType::Str) {
            let cast = self
                .builder
                .build_pointer_cast(value.into_pointer_value(), to_llvm.into_pointer_type(), "ptrcast")
                .map_err(|err| self.int_err(err))?;
            return Ok(cast.as_basic_value_enum());
        }
        Err(CodegenError::Unsupported {
            span: None,
            message: format!("cannot coerce between `{from}` and `{to}`"),
        })
    }

    
    
    

    
    
    
    
    fn current_fn_return_cty(&self) -> CType {
        self.current_fn_return.clone()
    }

    
    
    
    fn static_ctype(&self, e: &ast::Expr) -> CType {
        match e {
            ast::Expr::Literal(lit, _) => match lit {
                ast::LiteralExpr::Integer(_) => CType::IntLiteral,
                ast::LiteralExpr::Float(_) => CType::FloatLiteral,
                ast::LiteralExpr::Char(_) => CType::Char,
                ast::LiteralExpr::Bool(_) => CType::Bool,
                ast::LiteralExpr::String(_) => CType::Str,
                ast::LiteralExpr::Null => CType::Null,
            },
            ast::Expr::Var { name, .. } => self
                .lookup_var(name)
                .map(|(_, ty)| ty)
                .or_else(|| self.const_values.get(name).and_then(|expr| Some(self.static_ctype(expr))))
                .unwrap_or(CType::Void),
            ast::Expr::Call { .. } => CType::Void,
            ast::Expr::MethodCall { receiver, method, .. } => {
                let rt = self.static_ctype(receiver);
                let (struct_name, struct_args) = match &rt {
                    CType::Struct { name, args } => (name.clone(), args.clone()),
                    CType::Pointer { pointee, .. } => match &**pointee {
                        CType::Struct { name, args } => (name.clone(), args.clone()),
                        _ => return CType::Void,
                    },
                    _ => return CType::Void,
                };
                self.methods
                    .get(&struct_name)
                    .and_then(|m| m.get(method))
                    .and_then(|s| s.return_ty.clone())
                    .map(|ret| {
                        self.structs
                            .get(&struct_name)
                            .map(|info| {
                                let bindings: HashMap<String, CType> = info
                                    .generics
                                    .iter()
                                    .zip(struct_args.iter())
                                    .map(|(g, ty)| (g.clone(), ty.clone()))
                                    .collect();
                                ret.substitute(&bindings)
                            })
                            .unwrap_or(ret)
                    })
                    .unwrap_or(CType::Void)
            }
            ast::Expr::FieldAccess { base, field, .. } => {
                let bt = self.static_ctype(base);
                let (name, args) = match &bt {
                    CType::Struct { name, args } => (name.clone(), args.clone()),
                    CType::Pointer { pointee, .. } => match &**pointee {
                        CType::Struct { name, args } => (name.clone(), args.clone()),
                        _ => return CType::Void,
                    },
                    _ => return CType::Void,
                };
                self.field_type(&name, &args, field).unwrap_or(CType::Void)
            }
            ast::Expr::Index { base, .. } => match self.static_ctype(base) {
                CType::Pointer { pointee, .. } => (*pointee).clone(),
                CType::Array { element, .. } => (*element).clone(),
                _ => CType::Void,
            },
            ast::Expr::Unary { op, operand, .. } => match op {
                ast::UnaryOp::Neg => default_concrete(&self.static_ctype(operand)),
                ast::UnaryOp::Not => CType::Bool,
                ast::UnaryOp::BitNot => default_concrete(&self.static_ctype(operand)),
                ast::UnaryOp::Deref => match self.static_ctype(operand) {
                    CType::Pointer { pointee, .. } => (*pointee).clone(),
                    _ => CType::Void,
                },
                ast::UnaryOp::AddrOf | ast::UnaryOp::AddrOfMut => CType::Pointer {
                    pointee: Box::new(self.static_ctype(operand)),
                    mutable: *op == ast::UnaryOp::AddrOfMut,
                },
            },
            ast::Expr::Binary { op, lhs, .. } => {
                if matches!(
                    op,
                    ast::BinaryOp::Eq
                        | ast::BinaryOp::NotEq
                        | ast::BinaryOp::Lt
                        | ast::BinaryOp::Gt
                        | ast::BinaryOp::LtEq
                        | ast::BinaryOp::GtEq
                        | ast::BinaryOp::AndAnd
                        | ast::BinaryOp::OrOr
                ) {
                    CType::Bool
                } else {
                    default_concrete(&self.static_ctype(lhs))
                }
            }
            ast::Expr::Assign { target, .. } => self.static_ctype(target),
            ast::Expr::Cast { ty, .. } => self.ctype_from_ast(&*ty, e.span()).unwrap_or(CType::Void),
            ast::Expr::StructLiteral { name, .. } => {
                if self.structs.contains_key(name) {
                    CType::Struct { name: name.clone(), args: Vec::new() }
                } else {
                    CType::Void
                }
            }
            ast::Expr::Match { .. } | ast::Expr::Return { .. } | ast::Expr::Path { .. } => CType::Void,
            ast::Expr::ArrayRepeat { .. } | ast::Expr::ArrayLiteral { .. } => CType::Void,
            ast::Expr::Range { .. } => CType::Void,
            ast::Expr::SizeOf { .. } => CType::Usize,
        }
    }
}






fn struct_key(name: &str, args: &[CType]) -> String {
    if args.is_empty() {
        name.to_string()
    } else {
        format!("{name}${}", mangle_args(args))
    }
}



fn mangle_args(args: &[CType]) -> String {
    args.iter().map(mangle_c_type).collect::<Vec<_>>().join("_")
}


fn mangle_c_type(ty: &CType) -> String {
    match ty {
        CType::Void => "void".to_string(),
        CType::Bool => "bool".to_string(),
        CType::Char => "char".to_string(),
        CType::I8 => "i8".to_string(),
        CType::U8 => "u8".to_string(),
        CType::I16 => "i16".to_string(),
        CType::U16 => "u16".to_string(),
        CType::I32 => "i32".to_string(),
        CType::U32 => "u32".to_string(),
        CType::I64 => "i64".to_string(),
        CType::U64 => "u64".to_string(),
        CType::Isize => "isize".to_string(),
        CType::Usize => "usize".to_string(),
        CType::I128 => "i128".to_string(),
        CType::U128 => "u128".to_string(),
        CType::F32 => "f32".to_string(),
        CType::F64 => "f64".to_string(),
        CType::Str => "str".to_string(),
        CType::Null => "null".to_string(),
        CType::Pointer { pointee, .. } => format!("ptr_{}", mangle_c_type(pointee)),
        CType::Array { element, length } => format!("{}_x{length}", mangle_c_type(element)),
        CType::Generic(name) => format!("g_{name}"),
        CType::IntLiteral => "intlit".to_string(),
        CType::FloatLiteral => "floatlit".to_string(),
        CType::Struct { name, args } => {
            if args.is_empty() {
                name.clone()
            } else {
                format!("{name}_{}", mangle_args(args))
            }
        }
    }
}

fn method_llvm_name(struct_name: &str, method: &str) -> String {
    format!("{struct_name}__{method}")
}



fn default_concrete(ty: &CType) -> CType {
    match ty {
        CType::IntLiteral => CType::I64,
        CType::FloatLiteral => CType::F64,
        other => other.clone(),
    }
}

fn is_signed(ty: &CType) -> bool {
    !matches!(ty, CType::U8 | CType::U16 | CType::U32 | CType::U64 | CType::U128 | CType::Usize)
}

fn int_literal_value<'ctx>(context: &'ctx Context, value: i128) -> inkwell::values::IntValue<'ctx> {
    let (ty, words): (inkwell::types::IntType<'ctx>, Vec<u64>) =
        if value >= i64::MIN as i128 && value <= u64::MAX as i128 {
            (context.i64_type(), vec![value as u64])
        } else {
            (context.i128_type(), {
                let lo = value as u128 as u64;
                let hi = if value < 0 { u64::MAX } else { (value as u128 >> 64) as u64 };
                vec![lo, hi]
            })
        };
    ty.const_int_arbitrary_precision(&words)
}

fn int_predicate(op: ast::BinaryOp, concrete: &CType) -> IntPredicate {
    use ast::BinaryOp::*;
    let signed = is_signed(concrete);
    match op {
        Eq => IntPredicate::EQ,
        NotEq => IntPredicate::NE,
        Lt if signed => IntPredicate::SLT,
        Lt => IntPredicate::ULT,
        Gt if signed => IntPredicate::SGT,
        Gt => IntPredicate::UGT,
        LtEq if signed => IntPredicate::SLE,
        LtEq => IntPredicate::ULE,
        GtEq if signed => IntPredicate::SGE,
        GtEq => IntPredicate::UGE,
        _ => IntPredicate::EQ,
    }
}

fn float_predicate(op: ast::BinaryOp) -> FloatPredicate {
    use ast::BinaryOp::*;
    match op {
        Eq => FloatPredicate::OEQ,
        NotEq => FloatPredicate::ONE,
        Lt => FloatPredicate::OLT,
        Gt => FloatPredicate::OGT,
        LtEq => FloatPredicate::OLE,
        GtEq => FloatPredicate::OGE,
        _ => FloatPredicate::OEQ,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(src: &str) -> Result<String, CodegenError> {
        let tokens = crate::lexer::lex(src, "test.cp").expect("lex should succeed");
        let program = crate::parser::parse_tokens(tokens).expect("parse should succeed");
        let errors = crate::typechecker::check_program(&program);
        assert!(errors.is_empty(), "type check failed: {errors:?}");
        emit_llvm_ir(&program)
    }

    #[test]
    fn emits_main_function() {
        let ir = check("fn main() -> i32 { return 0; }").unwrap();
        assert!(ir.contains("define i32 @main"), "{ir}");
    }

    #[test]
    fn emits_arithmetic() {
        let ir = check("fn main(x: i32) -> i32 { let a: i32 = x + 1; return a * 2; }").unwrap();
        assert!(ir.contains("mul"), "{ir}");
        assert!(ir.contains("add"), "{ir}");
    }

    #[test]
    fn emits_while_and_if() {
        let ir = check(
            r#"
fn main() -> i32 {
    let mut i: i32 = 0;
    while i < 10 { i += 1; }
    if i == 10 { return i; }
    return 0;
}
"#,
        )
        .unwrap();
        assert!(ir.contains("icmp"), "{ir}");
        assert!(ir.contains("br i1"), "{ir}");
    }

    #[test]
    fn emits_struct_field_access() {
        let ir = check(
            r#"
struct Vector2 { x: f64, y: f64 }
fn main() -> f64 {
    let v: Vector2 = Vector2 { x: 1.5, y: 2.5 };
    return v.x;
}
"#,
        )
        .unwrap();
        assert!(ir.contains("define double @main"), "{ir}");
        assert!(ir.contains("getelementptr"), "{ir}");
    }

    #[test]
    fn emits_method_calls() {
        let ir = check(
            r#"
struct S { a: i32 }
impl S {
    fn new(x: i32) -> S { return S { a: x }; }
    fn get(self) -> i32 { return self.a; }
}
fn main() -> i32 {
    let s: S = S::new(7);
    return s.get();
}
"#,
        )
        .unwrap();
        assert!(ir.contains("define { i32 } @S__new"), "{ir}");
        assert!(ir.contains("define i32 @S__get"), "{ir}");
        assert!(ir.contains("call"), "{ir}");
    }

    #[test]
    fn destroy_is_only_emitted_when_called_explicitly() {
        let ir = check(
            r#"
struct Point { x: i32, y: i32 }
impl Point {
    fn new(x: i32, y: i32) -> Point { return Point { x: x, y: y }; }
    fn destroy(self) -> i32 { return self.x + self.y; }
}
fn main() -> i32 {
    let p: Point = Point(3, 4);
    let total: i32 = p.destroy();
    return total;
}
"#,
        )
        .unwrap();
        assert!(ir.contains("Point__destroy"), "{ir}");
        assert!(ir.contains("call"), "{ir}");
    }

    #[test]
    fn does_not_auto_call_destroy_on_scope_exit() {
        let ir = check(
            r#"
struct Point { x: i32, y: i32 }
impl Point {
    fn new(x: i32, y: i32) -> Point { return Point { x: x, y: y }; }
    fn destroy(self) -> i32 { return self.x + self.y; }
}
fn main() -> i32 {
    let p: Point = Point(3, 4);
    return 0;
}
"#,
        )
        .unwrap();
        
        
        assert!(!ir.contains("call i32 @Point__destroy"), "{ir}");
    }

    #[test]
    fn emits_operator_overload_calls() {
        let ir = check(
            r#"
struct Vec2 { x: i32, y: i32 }
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
    if a == b { return 0; }
    return 1;
}
"#,
        )
        .unwrap();
        assert!(ir.contains("Vec2__add"), "{ir}");
        assert!(ir.contains("Vec2__eq"), "{ir}");
        assert!(ir.contains("call"), "{ir}");
    }

    #[test]
    fn emits_inherited_struct_layout() {
        let ir = check(
            r#"
struct Base { x: i32 }
struct Derived : Base { y: i32 }
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
        )
        .unwrap();
        assert!(ir.contains("{ i32, i32 }"), "missing inherited field layout\n{ir}");
        assert!(ir.contains("Derived__new"), "missing Derived constructor\n{ir}");
    }

    #[test]
    fn emits_printf_builtin() {
        let ir = check("fn main() -> i32 { println(42); return 0; }").unwrap();
        assert!(ir.contains("declare i32 @printf"), "{ir}");
        assert!(ir.contains("call i32 (ptr, ...)"), "{ir}");
    }

    #[test]
    fn emits_pointer_indexing() {
        let ir = check("fn main(p: *const i32) -> i32 { return p[2]; }").unwrap();
        assert!(ir.contains("getelementptr"), "{ir}");
    }

    #[test]
    fn struct_field_assignment_reaches_local_storage() {
        let ir = check(
            r#"
struct T { a: i64, b: i64 }
fn main() -> i32 {
    let mut t: T = T { a: 1, b: 2 };
    t.a = 7;
    if t.a == 7 { return 0; }
    return 1;
}
"#,
        )
        .unwrap();
        assert!(
            !ir.contains("tmp.struct"),
            "struct field write must land in the variable's own storage, not a dead temporary\n{ir}"
        );
        assert!(ir.contains("store i64 7"), "{ir}");
    }

    #[test]
    fn indexed_pointer_struct_field_assignment_reaches_row() {
        let ir = check(
            r#"
struct T { a: i64, b: i64 }
fn main() -> i32 {
    let mut buf: *mut T = malloc(8 * sizeOf[T]) as *mut T;
    let r: T = T { a: 3, b: 4 };
    buf[0] = r;
    buf[0].b = 9;
    let g: T = buf[0];
    if g.b == 9 { return 0; }
    return 1;
}
extern fn malloc(size: usize) -> *mut void;
"#,
        )
        .unwrap();
        assert!(
            !ir.contains("tmp.struct"),
            "row struct field write must reach the pointed-to row, not a dead temporary\n{ir}"
        );
        assert!(ir.contains("store i64 9"), "{ir}");
    }

    #[test]
    fn generic_method_result_substitutes_concrete_args() {
        let ir = check(
            r#"
struct W[T] { v: T }
impl W[T] {
    fn wrap(v: T) -> W[T] { return W[T] { v: v }; }
    fn val(&self) -> T { return self.v; }
    fn set(&mut self, v: T) { self.v = v; }
}
fn main() -> i32 {
    let mut outer: W[W[i32]] = W::wrap(W::wrap(3 as i32));
    outer.set(W::wrap(5 as i32));
    let inner: W[i32] = outer.val();
    if inner.val() == 5 { return 0; }
    return 1;
}
"#,
        )
        .unwrap();
        assert!(ir.contains("define i32 @main"), "{ir}");
    }

    #[test]
    fn generic_free_function_monomorphizes() {
        let ir = check(
            r#"
fn id[T](x: T) -> T { return x; }
fn main() -> i32 {
    let a: i32 = id(2);
    let b: i32 = id(3);
    return id(a + b);
}
"#,
        )
        .unwrap();
        
        
        assert!(ir.contains("define i64 @\"id$fi64\"") || ir.contains("define i64 @id$fi64"), "{ir}");
        assert!(ir.contains("define i32 @\"id$fi32\"") || ir.contains("define i32 @id$fi32"), "{ir}");
        
        assert_eq!(ir.matches("id$fi64").count(), 3, "got:\n{ir}");
        
        assert_eq!(ir.matches("id$fi32").count(), 2, "got:\n{ir}");
    }

    #[test]
    fn generic_struct_and_methods_monomorphize() {
        let ir = check(
            r#"
struct Opt[T] { value: T }
impl Opt[T] {
    fn new(v: T) -> Opt[T] { return Opt { value: v }; }
    fn get(self) -> T { return self.value; }
}
fn main() -> i64 {
    let o: Opt[i32] = Opt[i32] { value: 7 };
    let p: Opt[i64] = Opt::new(9);
    return o.get() as i64 + p.get();
}
"#,
        )
        .unwrap();
        
        assert!(ir.contains("{ i32 }"), "missing Opt[i32] layout\n{ir}");
        assert!(ir.contains("{ i64 }"), "missing Opt[i64] layout\n{ir}");
        assert!(ir.contains("Opt__new$si64"), "missing new[i64] instance\n{ir}");
        assert!(ir.contains("Opt__get$si64"), "missing get[i64] instance\n{ir}");
        assert!(ir.contains("Opt__get$si32"), "missing get[i32] instance\n{ir}");
    }

    #[test]
    fn struct_literal_type_args_are_inferred_from_let() {
        
        
        let ir = check(
            r#"
struct Opt[T] { value: T }
fn main() -> i32 {
    let o: Opt[i32] = Opt { value: 7 };
    return o.value + 1;
}
"#,
        )
        .unwrap();
        assert!(ir.contains("{ i32 }"), "expected Opt[i32] layout, got:\n{ir}");
        assert!(ir.contains("add i32"), "expected i32 arithmetic, got:\n{ir}");
    }

    #[test]
    fn void_match_arm_prints_succeeds() {
        
        
        let ir = check(
            r#"
fn main() -> i32 {
    match 2 {
        2 => println(42),
        _ => return 1,
    }
    return 0;
}
"#,
        )
        .unwrap();
        assert!(ir.contains("call i32 (ptr, ...)"), "{ir}");
    }

    #[test]
    fn logical_and_or_short_circuit() {
        
        
        let ir = check(
            r#"
fn main(a: bool, b: bool) -> bool {
    return a && b && b || a;
}
"#,
        )
        .unwrap();
        assert!(ir.contains("phi"), "{ir}");
    }

    #[test]
    fn for_loop_over_string() {
        let ir = check(
            r#"
fn main() -> i32 {
    let mut count: i32 = 0;
    for c in "abc" { count += 1; }
    return count;
}
"#,
        )
        .unwrap();
        assert!(ir.contains("for.cond"), "{ir}");
    }

    #[test]
    fn pointer_deref_and_addrof() {
        let ir = check(
            r#"
fn main(x: i32) -> i32 {
    let p: *mut i32 = &mut x;
    return *p;
}
"#,
        )
        .unwrap();
        assert!(ir.contains("load"), "{ir}");
    }

    #[test]
    fn multi_arm_match_with_binding() {
        
        
        
        let ir = check(
            r#"
fn main(x: i32) -> i32 {
    return match x {
        0 => 10,
        n => n * 2,
    };
}
"#,
        )
        .unwrap();
        assert!(ir.contains("match.end"), "{ir}");
    }

    #[test]
    fn cast_between_int_widths() {
        let ir = check(
            r#"
fn main(x: i64) -> i32 {
    return x as i32;
}
"#,
        )
        .unwrap();
        assert!(ir.contains("trunc"), "{ir}");
    }
}