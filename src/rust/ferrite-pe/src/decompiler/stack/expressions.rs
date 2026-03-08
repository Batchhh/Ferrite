//! Free helper functions for expression construction and name resolution.

use super::*;

/// Helper to resolve a token or ResolvedName operand.
pub(super) fn resolve_operand_name(resolver: &MetadataResolver, operand: &Operand) -> String {
    match operand {
        Operand::Token(tok) => resolver.resolve_token(*tok),
        Operand::ResolvedName(name) => name.clone(),
        _ => String::new(),
    }
}

/// Helper to resolve call info from a token or resolved name.
/// For ResolvedName, expects "name|param_count|has_return|is_static" format.
pub(super) fn resolve_call_info_from_operand(
    resolver: &MetadataResolver,
    operand: &Operand,
) -> (String, usize, bool, bool) {
    match operand {
        Operand::Token(tok) => resolver.resolve_call_info(*tok),
        Operand::ResolvedName(info) => {
            let parts: Vec<&str> = info.splitn(4, '|').collect();
            if parts.len() == 4 {
                let name = parts[0].to_string();
                let param_count = parts[1].parse().unwrap_or(0);
                let has_return = parts[2] == "1";
                let is_static = parts[3] == "1";
                (name, param_count, has_return, is_static)
            } else {
                (info.clone(), 0, false, true)
            }
        }
        _ => (String::new(), 0, false, true),
    }
}

/// Split a qualified name like "Type::Member" into (type, member).
/// If no "::" is found, returns ("", full_name).
pub(super) fn split_qualified(name: &str) -> (String, String) {
    if let Some(pos) = name.rfind("::") {
        (name[..pos].to_string(), name[pos + 2..].to_string())
    } else {
        (String::new(), name.to_string())
    }
}

/// Strip the type prefix from an instance field name.
/// "Type::field" → "field", "field" → "field".
pub(super) fn strip_type_prefix(name: &str) -> String {
    if let Some(pos) = name.rfind("::") {
        name[pos + 2..].to_string()
    } else {
        name.to_string()
    }
}

/// Generate meaningful variable names from local type names.
/// Avoids collisions with parameter names and between locals.
pub(super) fn generate_local_names(type_names: &[String], params: &[String]) -> Vec<String> {
    let mut names = Vec::with_capacity(type_names.len());
    let mut counts: HashMap<String, usize> = HashMap::new();

    // Reserve parameter names to avoid collisions
    for p in params {
        counts.insert(p.clone(), 1);
    }

    for type_name in type_names {
        let base = type_to_var_name(type_name);
        let count = counts.entry(base.clone()).or_insert(0);
        *count += 1;
        if *count == 1 {
            names.push(base);
        } else {
            names.push(format!("{}{}", base, count));
        }
    }
    names
}

/// Convert a type name to a suitable variable name.
pub(super) fn type_to_var_name(type_name: &str) -> String {
    // Strip array brackets, generic parameters, and ref markers
    let clean = type_name
        .split(&['<', '[', '`', '&'][..])
        .next()
        .unwrap_or(type_name)
        .trim();

    // Strip "System." namespace prefix so e.g. "System.Boolean" matches "Boolean"
    let clean = clean.strip_prefix("System.").unwrap_or(clean);

    match clean {
        "int" | "Int32" | "Int16" | "Int64" | "long" | "short" | "byte" | "sbyte" | "uint"
        | "UInt32" | "UInt16" | "UInt64" | "ulong" | "ushort" | "float" | "Single" | "double"
        | "Double" | "decimal" | "Decimal" | "nint" | "nuint" | "IntPtr" | "UIntPtr" => {
            "num".into()
        }
        "bool" | "Boolean" => "flag".into(),
        "string" | "String" => "text".into(),
        "object" | "Object" => "obj".into(),
        "char" | "Char" => "c".into(),
        "void" | "Void" => "v".into(),
        "" => "obj".into(),
        _ => {
            // For other types, lowercase first letter
            let mut chars = clean.chars();
            if let Some(first) = chars.next() {
                let rest: String = chars.collect();
                format!("{}{}", first.to_lowercase(), rest)
            } else {
                "obj".into()
            }
        }
    }
}
