//! Type-level decompilation: emits a full C# type body from a `TypeDef`.

use crate::assembly::{Assembly, FieldDef, MethodDef, TypeDef, TypeKind};
use std::collections::HashSet;

use crate::decompiler::generation::header::{
    emit_custom_attributes, emit_enum_members, emit_type_header,
};
use crate::decompiler::generation::lambda_builder::{
    build_lambda_map, extract_field_initializers, is_backing_field, is_compiler_generated_type,
};
use crate::decompiler::generation::members::{
    emit_constructor, emit_field, emit_method, emit_property,
};
use crate::decompiler::INDENT;

/// Decompile a TypeDef to C# source code (no indentation).
pub(in crate::decompiler) fn decompile_type_def(td: &TypeDef, assembly: &Assembly) -> String {
    let mut out = String::new();

    emit_custom_attributes(&td.custom_attributes, &mut out, "");
    emit_type_header(td, &mut out);
    out.push_str("{\n");

    let accessor_tokens: HashSet<u32> = td
        .properties
        .iter()
        .flat_map(|p| {
            let mut tokens = Vec::new();
            if let Some(t) = p.getter_token {
                tokens.push(t);
            }
            if let Some(t) = p.setter_token {
                tokens.push(t);
            }
            tokens
        })
        .collect();

    if td.kind == TypeKind::Enum {
        emit_enum_members(td, &mut out);
    } else {
        emit_non_enum_members(td, assembly, &accessor_tokens, &mut out);
    }

    emit_nested_types(td, assembly, &mut out);

    out.push_str("}\n");
    out
}

/// Emit fields, properties, methods, and constructors for non-enum types.
fn emit_non_enum_members(
    td: &TypeDef,
    assembly: &Assembly,
    accessor_tokens: &HashSet<u32>,
    out: &mut String,
) {
    let lambda_map = build_lambda_map(td, assembly);
    let hoisted_inits = extract_field_initializers(td, assembly);
    let hoisted_field_names: HashSet<String> = hoisted_inits.keys().cloned().collect();

    let regular_fields: Vec<&FieldDef> = td
        .fields
        .iter()
        .filter(|f| !is_backing_field(&f.name))
        .collect();
    for field in &regular_fields {
        emit_custom_attributes(&field.custom_attributes, out, INDENT);
        let init = hoisted_inits.get(field.name.as_ref()).map(|s| s.as_str());
        emit_field(field, init, out);
    }

    if !td.properties.is_empty() {
        if !regular_fields.is_empty() {
            out.push('\n');
        }
        for (i, prop) in td.properties.iter().enumerate() {
            emit_property(prop, td, assembly, &lambda_map, out);
            if i < td.properties.len() - 1 {
                out.push('\n');
            }
        }
    }

    let regular_methods: Vec<&MethodDef> = td
        .methods
        .iter()
        .filter(|m| !accessor_tokens.contains(&m.token))
        .filter(|m| &*m.name != ".ctor" && &*m.name != ".cctor")
        .collect();

    if (!regular_fields.is_empty() || !td.properties.is_empty()) && !regular_methods.is_empty() {
        out.push('\n');
    }
    for (i, method) in regular_methods.iter().enumerate() {
        emit_custom_attributes(&method.custom_attributes, out, INDENT);
        emit_method(method, &td.name, &td.kind, assembly, &lambda_map, out);
        if i < regular_methods.len() - 1 {
            out.push('\n');
        }
    }

    emit_constructors(
        td,
        assembly,
        &hoisted_field_names,
        &lambda_map,
        out,
        &regular_fields,
        &regular_methods,
    );
}

/// Emit constructors (instance + static).
fn emit_constructors(
    td: &TypeDef,
    assembly: &Assembly,
    hoisted_field_names: &HashSet<String>,
    lambda_map: &crate::decompiler::patterns::LambdaMap,
    out: &mut String,
    regular_fields: &[&FieldDef],
    regular_methods: &[&MethodDef],
) {
    let ctors: Vec<&MethodDef> = td
        .methods
        .iter()
        .filter(|m| &*m.name == ".ctor" || &*m.name == ".cctor")
        .collect();
    if !ctors.is_empty() {
        if !regular_methods.is_empty() || !td.properties.is_empty() || !regular_fields.is_empty() {
            out.push('\n');
        }
        for (i, ctor) in ctors.iter().enumerate() {
            emit_custom_attributes(&ctor.custom_attributes, out, INDENT);
            emit_constructor(
                ctor,
                &td.name,
                assembly,
                hoisted_field_names,
                lambda_map,
                out,
            );
            if i < ctors.len() - 1 {
                out.push('\n');
            }
        }
    }
}

/// Emit nested types recursively, skipping compiler-generated ones.
fn emit_nested_types(td: &TypeDef, assembly: &Assembly, out: &mut String) {
    let visible_nested: Vec<&TypeDef> = td
        .nested_types
        .iter()
        .filter(|n| !is_compiler_generated_type(&n.name))
        .collect();
    if !visible_nested.is_empty() {
        out.push('\n');
        for (i, nested) in visible_nested.iter().enumerate() {
            let nested_code = decompile_type_def(nested, assembly);
            for line in nested_code.lines() {
                out.push_str(INDENT);
                out.push_str(line);
                out.push('\n');
            }
            if i < visible_nested.len() - 1 {
                out.push('\n');
            }
        }
    }
}
