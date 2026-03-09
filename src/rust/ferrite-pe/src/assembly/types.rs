use std::collections::HashMap;
use std::sync::Arc;

use crate::exception_handler::ExceptionHandler;
use crate::il::Instruction;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PeError {
    #[error("Failed to parse PE: {0}")]
    Parse(String),
    #[error("Not a valid PE file")]
    InvalidPe,
    #[error("Not a .NET assembly (no CLR header)")]
    NotDotNet,
}

/// The kind of a .NET type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeKind {
    Class,
    Interface,
    Struct,
    Enum,
    Delegate,
}

/// Parameter name + type info.
#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    pub type_name: String,
}

/// A custom attribute applied to a type, method, or field.
#[derive(Debug, Clone)]
pub struct CustomAttribute {
    pub name: String,
    pub arguments: Vec<String>,
}

/// Pre-parsed method body data.
#[derive(Debug, Clone)]
pub struct ParsedMethodBody {
    pub instructions: Vec<Instruction>,
    pub exception_handlers: Vec<ExceptionHandler>,
    pub locals: Vec<String>,
    pub max_stack: u16,
}

/// A parsed .NET method definition.
#[derive(Debug, Clone)]
pub struct MethodDef {
    pub token: u32,
    pub name: Arc<str>,
    pub flags: u16,
    pub impl_flags: u16,
    pub rva: u32,
    pub return_type: String,
    pub params: Vec<ParamInfo>,
    pub custom_attributes: Vec<CustomAttribute>,
    pub parsed_body: Option<ParsedMethodBody>,
}

/// A parsed .NET field definition.
#[derive(Debug, Clone)]
pub struct FieldDef {
    pub token: u32,
    pub name: Arc<str>,
    pub flags: u16,
    pub field_type: String,
    pub constant_value: Option<String>,
    pub custom_attributes: Vec<CustomAttribute>,
}

/// A parsed .NET property definition.
#[derive(Debug, Clone)]
pub struct PropertyDef {
    pub token: u32,
    pub name: Arc<str>,
    pub property_type: String,
    pub getter_token: Option<u32>,
    pub setter_token: Option<u32>,
    pub custom_attributes: Vec<CustomAttribute>,
}

/// A parsed .NET type definition.
#[derive(Debug, Clone)]
pub struct TypeDef {
    pub token: u32,
    pub name: Arc<str>,
    pub namespace: Arc<str>,
    pub kind: TypeKind,
    pub flags: u32,
    pub methods: Vec<MethodDef>,
    pub fields: Vec<FieldDef>,
    pub properties: Vec<PropertyDef>,
    pub nested_types: Vec<TypeDef>,
    pub base_type: Option<String>,
    pub interfaces: Vec<String>,
    pub custom_attributes: Vec<CustomAttribute>,
}

/// A reference to an external assembly.
#[derive(Debug, Clone)]
pub struct AssemblyRef {
    pub name: String,
    pub version: String,
}

/// Resolved information about a MemberRef table entry.
#[derive(Debug, Clone)]
pub struct MemberRefInfo {
    pub class_name: Arc<str>,
    pub name: Arc<str>,
    pub signature_blob: Vec<u8>,
}

/// Resolved information about a MethodSpec table entry.
#[derive(Debug, Clone)]
pub struct MethodSpecInfo {
    pub base_method_token: u32,
    pub generic_args: Vec<String>,
}

/// Call info for a MethodDef.
#[derive(Debug, Clone)]
pub struct MethodDefCallInfo {
    pub param_count: usize,
    pub has_return: bool,
    pub is_static: bool,
}

/// Deduplicates repeated strings by caching `Arc<str>` values.
///
/// Common strings like namespace names, primitive type names ("void", "int", "string"),
/// and frequently-referenced type names are shared across the entire assembly.
#[derive(Debug, Default)]
pub struct StringInterner {
    map: HashMap<String, Arc<str>>,
}

impl StringInterner {
    pub fn new() -> Self {
        let mut interner = Self::default();
        // Pre-seed with common C# primitive type names
        for s in [
            "void",
            "bool",
            "byte",
            "sbyte",
            "char",
            "short",
            "ushort",
            "int",
            "uint",
            "long",
            "ulong",
            "float",
            "double",
            "string",
            "object",
            "decimal",
            "IntPtr",
            "UIntPtr",
            "TypedReference",
            "delegate*",
            "void*",
            "System.Void",
            "System.Object",
            "System.String",
            "System.Boolean",
            "System.Int32",
            "System.Int64",
            "System.Single",
            "System.Double",
            "System.Byte",
            "System.Char",
            "System.Enum",
            "System.ValueType",
            "System.MulticastDelegate",
            "System.Attribute",
            "",
        ] {
            let arc: Arc<str> = Arc::from(s);
            interner.map.insert(s.to_string(), arc);
        }
        interner
    }

    /// Intern a string, returning a shared `Arc<str>`.
    pub fn intern(&mut self, s: &str) -> Arc<str> {
        if let Some(existing) = self.map.get(s) {
            Arc::clone(existing)
        } else {
            let arc: Arc<str> = Arc::from(s);
            self.map.insert(s.to_string(), Arc::clone(&arc));
            arc
        }
    }

    /// Intern a `String`, consuming it to avoid an extra allocation when it's new.
    pub fn intern_string(&mut self, s: String) -> Arc<str> {
        if let Some(existing) = self.map.get(&s) {
            Arc::clone(existing)
        } else {
            let arc: Arc<str> = Arc::from(s.as_str());
            self.map.insert(s, Arc::clone(&arc));
            arc
        }
    }
}

/// A fully parsed .NET assembly.
#[derive(Debug, Clone)]
pub struct Assembly {
    pub name: String,
    pub version: String,
    pub module_name: String,
    pub target_framework: String,
    pub types: Vec<TypeDef>,
    pub assembly_refs: Vec<AssemblyRef>,

    // Flat name/info tables indexed by metadata row (0-based).
    // Used for resolving token operands during IL decoding.
    pub type_def_names: Vec<Arc<str>>,
    pub method_def_names: Vec<Arc<str>>,
    pub field_def_names: Vec<Arc<str>>,
    pub type_ref_names: Vec<Arc<str>>,
    pub member_ref_infos: Vec<MemberRefInfo>,
    pub type_spec_names: Vec<Arc<str>>,
    pub method_spec_infos: Vec<MethodSpecInfo>,
    pub method_def_call_infos: Vec<MethodDefCallInfo>,
    pub us_heap_data: Vec<u8>,

    pub user_strings: HashMap<u32, String>,
    pub(crate) next_string_token: u32,
}

impl Assembly {
    /// Get call info for a MethodDef by row number (1-based).
    pub fn get_method_def_call_info(&self, row: usize) -> (usize, bool, bool) {
        if let Some(info) = self.method_def_call_infos.get(row.wrapping_sub(1)) {
            (info.param_count, info.has_return, info.is_static)
        } else {
            (0, false, true)
        }
    }

    /// Allocate a synthetic string token and store the resolved string.
    pub(crate) fn intern_string(&mut self, s: String) -> u32 {
        let token = 0x70000000 | self.next_string_token;
        self.user_strings.insert(token, s);
        self.next_string_token += 1;
        token
    }
}
