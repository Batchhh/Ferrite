use crate::assembly::{CustomAttribute, FieldDef, TypeDef, TypeKind};
use crate::decompiler::INDENT;

/// Emit the type declaration header (visibility, modifiers, keyword, name, base list) into `out`.
pub(in crate::decompiler) fn emit_type_header(td: &TypeDef, out: &mut String) {
    let flags = td.flags;

    let vis = type_visibility(flags);
    if !vis.is_empty() {
        out.push_str(vis);
        out.push(' ');
    }

    // A type is `static` when both `abstract` (0x80) and `sealed` (0x100) are set.
    let is_abstract = (flags & 0x0080) != 0;
    let is_sealed = (flags & 0x0100) != 0;
    let is_static = is_abstract && is_sealed && td.kind == TypeKind::Class;

    if is_static {
        out.push_str("static ");
    } else {
        if is_abstract && td.kind == TypeKind::Class {
            out.push_str("abstract ");
        }
        if is_sealed && td.kind == TypeKind::Class {
            out.push_str("sealed ");
        }
    }

    out.push_str(type_kind_keyword(&td.kind));
    out.push(' ');
    out.push_str(&td.name);

    let mut inheritance: Vec<&str> = Vec::new();
    if let Some(ref base) = td.base_type {
        // Skip System.Object, System.ValueType, System.Enum, System.MulticastDelegate
        if !is_implicit_base(base, &td.kind) {
            inheritance.push(base.as_str());
        }
    }
    for iface in &td.interfaces {
        inheritance.push(iface.as_str());
    }
    if !inheritance.is_empty() {
        out.push('\n');
        out.push_str(INDENT);
        out.push_str(": ");
        for (i, item) in inheritance.iter().enumerate() {
            if i > 0 {
                out.push_str(",\n");
                out.push_str(INDENT);
                out.push_str("  ");
            }
            out.push_str(item);
        }
        out.push('\n');
    } else {
        out.push('\n');
    }
}

/// Map raw type flags to the C# visibility keyword (empty string for internal/default).
pub(in crate::decompiler) fn type_visibility(flags: u32) -> &'static str {
    match flags & 0x07 {
        0x00 => "", // not public (internal)
        0x01 => "public",
        0x02 => "public",             // nested public
        0x03 => "private",            // nested private
        0x04 => "protected",          // nested family
        0x05 => "internal",           // nested assembly
        0x06 => "protected internal", // nested family-or-assembly
        0x07 => "private protected",  // nested family-and-assembly
        _ => "",
    }
}

/// Return the C# keyword for a type kind (`class`, `interface`, `struct`, `enum`).
pub(in crate::decompiler) fn type_kind_keyword(kind: &TypeKind) -> &'static str {
    match kind {
        TypeKind::Class => "class",
        TypeKind::Interface => "interface",
        TypeKind::Struct => "struct",
        TypeKind::Enum => "enum",
        TypeKind::Delegate => "class", // delegates are classes at the IL level
    }
}

/// Return `true` if `base` is the implicit base class for `kind` and should be suppressed in output.
pub(in crate::decompiler) fn is_implicit_base(base: &str, kind: &TypeKind) -> bool {
    match kind {
        TypeKind::Class | TypeKind::Delegate => {
            base == "System.Object"
                || base == "Object"
                || base == "System.MulticastDelegate"
                || base == "MulticastDelegate"
        }
        TypeKind::Struct => base == "System.ValueType" || base == "ValueType",
        TypeKind::Enum => base == "System.Enum" || base == "Enum",
        TypeKind::Interface => true, // interfaces don't show base type
    }
}

/// Emit `[AttributeName(args)]` lines for each attribute, prefixed with `indent`.
pub(in crate::decompiler) fn emit_custom_attributes(
    attrs: &[CustomAttribute],
    out: &mut String,
    indent: &str,
) {
    for attr in attrs {
        out.push_str(indent);
        out.push('[');
        out.push_str(&attr.name);
        if !attr.arguments.is_empty() {
            out.push('(');
            for (i, arg) in attr.arguments.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                out.push_str(arg);
            }
            out.push(')');
        }
        out.push_str("]\n");
    }
}

/// Emit enum member declarations (static literal fields) into `out`.
pub(in crate::decompiler) fn emit_enum_members(td: &TypeDef, out: &mut String) {
    let enum_members: Vec<&FieldDef> = td
        .fields
        .iter()
        .filter(|f| {
            let is_static = (f.flags & 0x0010) != 0;
            let is_literal = (f.flags & 0x0040) != 0;
            is_static && is_literal
        })
        .collect();

    for (i, field) in enum_members.iter().enumerate() {
        out.push_str(INDENT);
        out.push_str(&field.name);
        if let Some(ref val) = field.constant_value {
            out.push_str(" = ");
            out.push_str(val);
        }
        if i < enum_members.len() - 1 {
            out.push(',');
        }
        out.push('\n');
    }
}
