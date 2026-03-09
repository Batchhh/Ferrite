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
mod generation;
pub(crate) mod patterns;
pub(crate) mod resolver;
pub(crate) mod stack;

use crate::assembly::{Assembly, PeError, TypeDef};

use ast::type_emit::decompile_type_def;

pub(crate) const INDENT: &str = "    ";

/// Configuration for decompilation limits.
pub struct DecompilerConfig {
    pub max_statements: usize,
    pub max_depth: usize,
}

impl Default for DecompilerConfig {
    fn default() -> Self {
        Self {
            max_statements: 100_000,
            max_depth: 50,
        }
    }
}

/// Decompile a type definition to C# source code.
///
/// `type_token` is a TypeDef token (0x02XXXXXX).
pub fn decompile_type(assembly: &Assembly, type_token: u32) -> Result<String, PeError> {
    decompile_type_with_config(assembly, type_token, &DecompilerConfig::default())
}

/// Decompile a type with explicit limits.
pub fn decompile_type_with_config(
    assembly: &Assembly,
    type_token: u32,
    config: &DecompilerConfig,
) -> Result<String, PeError> {
    let td = find_type_by_token(assembly, type_token)
        .ok_or_else(|| PeError::Parse(format!("TypeDef token 0x{:08X} not found", type_token)))?;
    let result = decompile_type_def(td, assembly);
    let stmt_count = result.lines().count();
    if stmt_count > config.max_statements {
        return Err(PeError::Parse(format!(
            "Decompilation exceeded statement limit ({} > {})",
            stmt_count, config.max_statements
        )));
    }
    Ok(result)
}

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
