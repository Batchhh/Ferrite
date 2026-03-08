use crate::types::*;

/// Map CLR type visibility flags (ECMA-335 §II.23.1.15) to [`Visibility`].
pub(crate) fn decode_type_visibility(flags: u32) -> Visibility {
    match flags & 0x07 {
        0x00 => Visibility::Internal,          // NotPublic
        0x01 => Visibility::Public,            // Public
        0x02 => Visibility::Public,            // NestedPublic
        0x03 => Visibility::Private,           // NestedPrivate
        0x04 => Visibility::Protected,         // NestedFamily
        0x05 => Visibility::Internal,          // NestedAssembly
        0x06 => Visibility::ProtectedInternal, // NestedFamORAssem
        0x07 => Visibility::PrivateProtected,  // NestedFamANDAssem
        _ => Visibility::Private,
    }
}

/// Decode CLR type attribute flags to [`TypeAttributes`].
///
/// A type is considered `static` when both `Abstract` (0x80) and `Sealed` (0x100) are set.
pub(crate) fn decode_type_attributes(flags: u32) -> TypeAttributes {
    let is_abstract = flags & 0x80 != 0; // Abstract
    let is_sealed = flags & 0x100 != 0; // Sealed
    let is_static = is_abstract && is_sealed; // static = abstract + sealed in CLR
    TypeAttributes {
        visibility: decode_type_visibility(flags),
        is_abstract: is_abstract && !is_static,
        is_sealed: is_sealed && !is_static,
        is_static,
    }
}

/// Map CLR method visibility flags (ECMA-335 §II.23.1.10) to [`Visibility`].
pub(crate) fn decode_method_visibility(flags: u16) -> Visibility {
    match flags & 0x07 {
        0x01 => Visibility::Private,
        0x02 => Visibility::PrivateProtected,  // FamANDAssem
        0x03 => Visibility::Internal,          // Assembly
        0x04 => Visibility::Protected,         // Family
        0x05 => Visibility::ProtectedInternal, // FamORAssem
        0x06 => Visibility::Public,
        _ => Visibility::Private, // CompilerControlled / PrivateScope
    }
}

/// Decode CLR method attribute flags to [`MethodAttributes`].
pub(crate) fn decode_method_attributes(flags: u16, name: &str) -> MethodAttributes {
    MethodAttributes {
        visibility: decode_method_visibility(flags),
        is_static: flags & 0x10 != 0,
        is_virtual: flags & 0x40 != 0,
        is_abstract: flags & 0x400 != 0,
        is_final: flags & 0x20 != 0,
        is_constructor: name == ".ctor" || name == ".cctor",
    }
}

/// Map CLR field visibility flags (ECMA-335 §II.23.1.5) to [`Visibility`].
pub(crate) fn decode_field_visibility(flags: u16) -> Visibility {
    match flags & 0x07 {
        0x01 => Visibility::Private,
        0x02 => Visibility::PrivateProtected,
        0x03 => Visibility::Internal,
        0x04 => Visibility::Protected,
        0x05 => Visibility::ProtectedInternal,
        0x06 => Visibility::Public,
        _ => Visibility::Private,
    }
}

/// Decode CLR field attribute flags to [`FieldAttributes`].
pub(crate) fn decode_field_attributes(flags: u16) -> FieldAttributes {
    FieldAttributes {
        visibility: decode_field_visibility(flags),
        is_static: flags & 0x10 != 0,
        is_init_only: flags & 0x20 != 0,
        is_literal: flags & 0x40 != 0,
    }
}
