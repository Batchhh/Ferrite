#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum FerriteError {
    #[error("Failed to open file: {message}")]
    IoError { message: String },
    #[error("Parse error: {message}")]
    ParseError { message: String },
    #[error("Not found: {message}")]
    NotFound { message: String },
    #[error("Decompilation failed: {message}")]
    DecompilationFailed { message: String },
    #[error("Invalid token: {message}")]
    InvalidToken { message: String },
    #[error("Timeout: {message}")]
    Timeout { message: String },
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum TypeKind {
    Class,
    Interface,
    Struct,
    Enum,
    Delegate,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum MemberKind {
    Method,
    Field,
    Property,
    Event,
}

#[derive(Debug, Clone, uniffi::Enum)]
pub enum Visibility {
    Public,
    Private,
    Internal,
    Protected,
    ProtectedInternal,
    PrivateProtected,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct TypeAttributes {
    pub visibility: Visibility,
    pub is_abstract: bool,
    pub is_sealed: bool,
    pub is_static: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct MethodAttributes {
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_virtual: bool,
    pub is_abstract: bool,
    pub is_final: bool,
    pub is_constructor: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct FieldAttributes {
    pub visibility: Visibility,
    pub is_static: bool,
    pub is_init_only: bool,
    pub is_literal: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ParameterInfo {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct AttributeInfo {
    pub name: String,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct PropertyInfo {
    pub name: String,
    pub token: u32,
    pub property_type: String,
    pub getter_token: Option<u32>,
    pub setter_token: Option<u32>,
    pub attributes_list: Vec<AttributeInfo>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct MemberInfo {
    pub name: String,
    pub kind: MemberKind,
    pub token: u32,
    pub signature: String,
    pub method_attributes: Option<MethodAttributes>,
    pub field_attributes: Option<FieldAttributes>,
    pub return_type: String,
    pub parameters: Vec<ParameterInfo>,
    pub attributes_list: Vec<AttributeInfo>,
    pub field_type: String,
    pub constant_value: Option<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct TypeInfo {
    pub name: String,
    pub full_name: String,
    pub kind: TypeKind,
    pub token: u32,
    pub namespace: String,
    pub attributes: TypeAttributes,
    pub members: Vec<MemberInfo>,
    pub properties: Vec<PropertyInfo>,
    pub nested_types: Vec<TypeInfo>,
    pub base_type: Option<String>,
    pub interfaces: Vec<String>,
    pub attributes_list: Vec<AttributeInfo>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct NamespaceInfo {
    pub name: String,
    pub types: Vec<TypeInfo>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct AssemblyInfo {
    pub name: String,
    pub version: String,
    pub target_framework: String,
    pub namespaces: Vec<NamespaceInfo>,
    pub assembly_references: Vec<String>,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct LoadedAssemblyEntry {
    pub id: String,
    pub file_path: String,
    pub info: AssemblyInfo,
}

// --- Lazy loading summary types ---

/// Lightweight assembly summary returned on load (no type details).
#[derive(Debug, Clone, uniffi::Record)]
pub struct AssemblySummary {
    pub id: String,
    pub file_path: String,
    pub name: String,
    pub version: String,
    pub target_framework: String,
    pub namespaces: Vec<NamespaceSummary>,
    pub assembly_references: Vec<String>,
}

/// Namespace with type count (types loaded on demand).
#[derive(Debug, Clone, uniffi::Record)]
pub struct NamespaceSummary {
    pub name: String,
    pub type_count: u32,
}

/// Lightweight type summary (no members/attributes).
#[derive(Debug, Clone, uniffi::Record)]
pub struct TypeSummary {
    pub name: String,
    pub full_name: String,
    pub kind: TypeKind,
    pub token: u32,
    pub namespace: String,
    pub attributes: TypeAttributes,
    pub member_count: u32,
    pub property_count: u32,
    pub nested_type_count: u32,
    pub base_type: Option<String>,
    pub interfaces: Vec<String>,
}

/// A single searchable item (type or member) for building the search index.
#[derive(Debug, Clone, uniffi::Record)]
pub struct SearchableItem {
    pub name: String,
    pub full_name: String,
    pub kind: SearchableKind,
    pub token: u32,
    pub parent_token: Option<u32>,
}

/// Kind of searchable item.
#[derive(Debug, Clone, uniffi::Enum)]
pub enum SearchableKind {
    Class,
    Interface,
    Struct,
    Enum,
    Delegate,
    Method,
    Field,
    Property,
    Constant,
}
