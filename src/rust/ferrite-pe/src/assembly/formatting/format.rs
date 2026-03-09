use dotnetdll::prelude::*;

use super::super::TypeKind;
use super::format_types::format_type_source;

/// Simplify explicit interface implementation names like
/// `System.IEquatable<System.IntPtr>.Equals` → `IEquatable<IntPtr>.Equals`.
pub(in crate::assembly) fn simplify_method_name(name: &str) -> String {
    // Only process names that look like explicit interface implementations (contain a dot
    // before the final method name, outside of generic brackets).
    // Regular method names like "Equals" or ".ctor" pass through unchanged.
    let mut depth = 0i32;
    let mut last_dot_outside = None;
    for (i, ch) in name.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            '.' if depth == 0 => last_dot_outside = Some(i),
            _ => {}
        }
    }
    match last_dot_outside {
        // No dots or only ".ctor"/".cctor" — return as-is
        None => name.to_string(),
        Some(0) => name.to_string(),
        Some(dot_pos) => {
            let interface_part = &name[..dot_pos];
            let method_part = &name[dot_pos + 1..];
            let simple_interface = strip_namespaces_from_type(interface_part);
            format!("{}.{}", simple_interface, method_part)
        }
    }
}

/// Strip namespace prefixes from a type expression, handling generics.
/// e.g. `System.IEquatable<System.IntPtr>` → `IEquatable<IntPtr>`
pub(in crate::assembly) fn strip_namespaces_from_type(s: &str) -> String {
    if let Some(open) = s.find('<') {
        let base = s[..open].rsplit('.').next().unwrap_or(&s[..open]);
        let inner = &s[open + 1..s.len() - 1]; // inside < >
        let args: Vec<String> = split_generic_args(inner)
            .iter()
            .map(|a| strip_namespaces_from_type(a.trim()))
            .collect();
        format!("{}<{}>", base, args.join(", "))
    } else {
        s.rsplit('.').next().unwrap_or(s).to_string()
    }
}

/// Split generic arguments respecting nested angle brackets.
pub(in crate::assembly) fn split_generic_args(s: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut depth = 0i32;
    let mut start = 0;
    for (i, ch) in s.char_indices() {
        match ch {
            '<' => depth += 1,
            '>' => depth -= 1,
            ',' if depth == 0 => {
                args.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    args.push(&s[start..]);
    args
}

pub(in crate::assembly) fn format_type_full_name(typedef: &TypeDefinition) -> String {
    if let Some(ref ns) = typedef.namespace {
        format!("{}.{}", ns, typedef.name)
    } else {
        typedef.name.to_string()
    }
}

pub(in crate::assembly) fn determine_type_kind(
    typedef: &TypeDefinition,
    res: &Resolution,
) -> TypeKind {
    if matches!(
        typedef.flags.kind,
        dotnetdll::resolved::types::Kind::Interface
    ) {
        return TypeKind::Interface;
    }
    if let Some(ref extends) = typedef.extends {
        let base_name = format_type_source(extends, res);
        if base_name == "System.Enum" || base_name == "Enum" {
            return TypeKind::Enum;
        }
        if base_name == "System.ValueType" || base_name == "ValueType" {
            return TypeKind::Struct;
        }
        if base_name == "System.MulticastDelegate" || base_name == "MulticastDelegate" {
            return TypeKind::Delegate;
        }
    }
    TypeKind::Class
}
