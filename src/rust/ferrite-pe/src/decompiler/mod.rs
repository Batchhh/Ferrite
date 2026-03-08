//! Decompiler module — converts IL bytecode back to C# source code.
//!
//! This module provides:
//! - `resolver` — Metadata token resolution (tokens → human-readable names)
//! - `ast` — C# AST types
//! - `stack` — Stack simulation (IL → AST)
//! - `emit` — C# code emission (AST → text)
//! - `decompile_type` — Type-level decompilation orchestrator

pub(crate) mod ast;
pub(crate) mod control_flow;
pub(crate) mod emit;
pub(crate) mod patterns;
pub(crate) mod resolver;
pub(crate) mod stack;

mod header;
mod lambda_builder;
mod members;
mod methods;

use crate::assembly::{Assembly, FieldDef, MethodDef, PeError, TypeDef, TypeKind};
use std::collections::HashSet;

use header::{emit_custom_attributes, emit_enum_members, emit_type_header};
use lambda_builder::{
    build_lambda_map, extract_field_initializers, is_backing_field, is_compiler_generated_type,
};
use members::{emit_constructor, emit_field, emit_method, emit_property};

pub(crate) const INDENT: &str = "    ";

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Decompile a type definition to C# source code.
///
/// `type_token` is a TypeDef token (0x02XXXXXX).
pub fn decompile_type(assembly: &Assembly, type_token: u32) -> Result<String, PeError> {
    let td = find_type_by_token(assembly, type_token)
        .ok_or_else(|| PeError::Parse(format!("TypeDef token 0x{:08X} not found", type_token)))?;
    Ok(decompile_type_def(td, assembly))
}

/// Decompile a TypeDef to C# source code (no indentation).
fn decompile_type_def(td: &TypeDef, assembly: &Assembly) -> String {
    let mut out = String::new();

    emit_custom_attributes(&td.custom_attributes, &mut out, "");
    emit_type_header(td, &mut out);
    out.push_str("{\n");

    // Collect property accessor tokens so we can skip them in the methods section
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
        // Build lambda map from compiler-generated nested types (e.g. <>c)
        let lambda_map = build_lambda_map(td, assembly);

        // Pre-process: extract field initializers that can be hoisted from constructors
        let hoisted_inits = extract_field_initializers(td, assembly);
        let hoisted_field_names: HashSet<String> = hoisted_inits.keys().cloned().collect();

        let regular_fields: Vec<&FieldDef> = td
            .fields
            .iter()
            .filter(|f| !is_backing_field(&f.name))
            .collect();
        for field in &regular_fields {
            emit_custom_attributes(&field.custom_attributes, &mut out, INDENT);
            let init = hoisted_inits.get(field.name.as_ref()).map(|s| s.as_str());
            emit_field(field, init, &mut out);
        }

        if !td.properties.is_empty() {
            if !regular_fields.is_empty() {
                out.push('\n');
            }
            for (i, prop) in td.properties.iter().enumerate() {
                emit_property(prop, td, assembly, &lambda_map, &mut out);
                if i < td.properties.len() - 1 {
                    out.push('\n');
                }
            }
        }

        // Methods (excluding property accessors and constructors — ctors emitted separately)
        let regular_methods: Vec<&MethodDef> = td
            .methods
            .iter()
            .filter(|m| !accessor_tokens.contains(&m.token))
            .filter(|m| &*m.name != ".ctor" && &*m.name != ".cctor")
            .collect();

        if (!regular_fields.is_empty() || !td.properties.is_empty()) && !regular_methods.is_empty()
        {
            out.push('\n');
        }
        for (i, method) in regular_methods.iter().enumerate() {
            emit_custom_attributes(&method.custom_attributes, &mut out, INDENT);
            emit_method(method, &td.name, &td.kind, assembly, &lambda_map, &mut out);
            if i < regular_methods.len() - 1 {
                out.push('\n');
            }
        }

        let ctors: Vec<&MethodDef> = td
            .methods
            .iter()
            .filter(|m| &*m.name == ".ctor" || &*m.name == ".cctor")
            .collect();
        if !ctors.is_empty() {
            if !regular_methods.is_empty()
                || !td.properties.is_empty()
                || !regular_fields.is_empty()
            {
                out.push('\n');
            }
            for (i, ctor) in ctors.iter().enumerate() {
                emit_custom_attributes(&ctor.custom_attributes, &mut out, INDENT);
                emit_constructor(
                    ctor,
                    &td.name,
                    assembly,
                    &hoisted_field_names,
                    &lambda_map,
                    &mut out,
                );
                if i < ctors.len() - 1 {
                    out.push('\n');
                }
            }
        }
    }

    // Nested types — recursively decompile with indentation (skip compiler-generated)
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

    out.push_str("}\n");

    out
}

// ---------------------------------------------------------------------------
// Type lookup
// ---------------------------------------------------------------------------

fn find_type_by_token(assembly: &Assembly, token: u32) -> Option<&TypeDef> {
    for td in &assembly.types {
        if td.token == token {
            return Some(td);
        }
        if let Some(nested) = find_nested_type(td, token) {
            return Some(nested);
        }
    }
    None
}

fn find_nested_type(td: &TypeDef, token: u32) -> Option<&TypeDef> {
    for nested in &td.nested_types {
        if nested.token == token {
            return Some(nested);
        }
        if let Some(found) = find_nested_type(nested, token) {
            return Some(found);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests;
