use crate::assembly::PeError;
use crate::assembly::{Assembly, MethodDef, TypeKind};
use crate::decompiler::ast::Statement;
use crate::decompiler::control_flow::ControlFlowAnalyzer;
use crate::decompiler::emit::emit_statements;
use crate::decompiler::patterns;
use crate::decompiler::resolver::MetadataResolver;
use crate::decompiler::INDENT;
use std::collections::HashSet;

/// Emit a method declaration and body into `out`.
pub(in crate::decompiler) fn emit_method(
    method: &MethodDef,
    type_name: &str,
    type_kind: &TypeKind,
    assembly: &Assembly,
    lambda_map: &patterns::LambdaMap,
    out: &mut String,
) {
    out.push_str(INDENT);

    let vis = method_visibility(method.flags);
    if !vis.is_empty() {
        out.push_str(vis);
        out.push(' ');
    }

    if (method.flags & 0x0010) != 0 {
        out.push_str("static ");
    }
    if (method.flags & 0x0400) != 0 && (method.flags & 0x0040) == 0 {
        out.push_str("abstract ");
    } else if (method.flags & 0x0040) != 0 && (method.flags & 0x0020) == 0 {
        // virtual but not final — could be virtual or override
        // For simplicity, emit "virtual" (proper override detection needs base method lookup)
        if *type_kind != TypeKind::Interface {
            out.push_str("virtual ");
        }
    }

    out.push_str(&method.return_type);
    out.push(' ');
    out.push_str(&method.name);
    emit_params(&method.params, out);

    let is_abstract_method = (method.flags & 0x0400) != 0;
    let is_extern = (method.impl_flags & 0x1000) != 0 || (method.flags & 0x2000) != 0;

    if is_abstract_method || method.rva == 0 || is_extern || *type_kind == TypeKind::Interface {
        out.push_str(";\n");
    } else {
        out.push('\n');
        out.push_str(INDENT);
        out.push_str("{\n");
        match decompile_method_body(method, assembly, 2, type_name, lambda_map) {
            Ok(body) => out.push_str(&body),
            Err(_) => {
                out.push_str(INDENT);
                out.push_str(INDENT);
                out.push_str("/* decompilation failed */\n");
            }
        }
        out.push_str(INDENT);
        out.push_str("}\n");
    }
}

/// Emit a constructor declaration and body into `out`.
///
/// Field assignments that were hoisted to inline initializers are filtered from the body.
pub(in crate::decompiler) fn emit_constructor(
    method: &MethodDef,
    type_name: &str,
    assembly: &Assembly,
    hoisted_fields: &HashSet<String>,
    lambda_map: &patterns::LambdaMap,
    out: &mut String,
) {
    out.push_str(INDENT);

    let is_static_ctor = &*method.name == ".cctor";

    if is_static_ctor {
        out.push_str("static ");
        out.push_str(type_name);
        out.push_str("()");
    } else {
        let vis = method_visibility(method.flags);
        if !vis.is_empty() {
            out.push_str(vis);
            out.push(' ');
        }
        out.push_str(type_name);
        emit_params(&method.params, out);
    }

    if method.rva == 0 {
        out.push_str(";\n");
    } else {
        out.push('\n');
        out.push_str(INDENT);
        out.push_str("{\n");

        // If we have hoisted fields, build statements and filter them
        if !hoisted_fields.is_empty() && !is_static_ctor {
            if let Some(stmts) = super::lambda_builder::build_method_statements(
                method, assembly, type_name, lambda_map,
            ) {
                let body = emit_constructor_body_filtered(&stmts, hoisted_fields, 2);
                out.push_str(&body);
            } else {
                out.push_str(INDENT);
                out.push_str(INDENT);
                out.push_str("/* decompilation failed */\n");
            }
        } else {
            match decompile_method_body(method, assembly, 2, type_name, lambda_map) {
                Ok(body) => out.push_str(&body),
                Err(_) => {
                    out.push_str(INDENT);
                    out.push_str(INDENT);
                    out.push_str("/* decompilation failed */\n");
                }
            }
        }

        out.push_str(INDENT);
        out.push_str("}\n");
    }
}

/// Emit a parameter list `(type name, ...)` into `out`, aligning multi-line params.
pub(in crate::decompiler) fn emit_params(params: &[crate::assembly::ParamInfo], out: &mut String) {
    if params.is_empty() {
        out.push_str("()");
        return;
    }

    if params.len() > 1 {
        // Compute alignment column: position right after "("
        let current_line_len = out.rfind('\n').map_or(out.len(), |pos| out.len() - pos - 1);
        let align_col = current_line_len + 1; // +1 for the "(" itself
        let align_pad: String = " ".repeat(align_col);

        out.push('(');
        for (i, param) in params.iter().enumerate() {
            if i > 0 {
                out.push_str(&align_pad);
            }
            out.push_str(&param.type_name);
            out.push(' ');
            out.push_str(&param.name);
            if i < params.len() - 1 {
                out.push_str(",\n");
            } else {
                out.push(')');
            }
        }
    } else {
        out.push('(');
        out.push_str(&params[0].type_name);
        out.push(' ');
        out.push_str(&params[0].name);
        out.push(')');
    }
}

/// Map raw method flags to the C# visibility keyword.
pub(in crate::decompiler) fn method_visibility(flags: u16) -> &'static str {
    match flags & 0x0007 {
        0x0001 => "private",
        0x0002 => "private protected",
        0x0003 => "internal",
        0x0004 => "protected",
        0x0005 => "protected internal",
        0x0006 => "public",
        _ => "private",
    }
}

/// Decompile a method body to indented C# source.
///
/// Runs control-flow analysis, applies pattern transforms, then emits statements.
pub(in crate::decompiler) fn decompile_method_body(
    method: &MethodDef,
    assembly: &Assembly,
    indent: usize,
    enclosing_type: &str,
    lambda_map: &patterns::LambdaMap,
) -> Result<String, PeError> {
    let parsed_body = method
        .parsed_body
        .as_ref()
        .ok_or_else(|| PeError::Parse("method has no parsed body".into()))?;

    let params: Vec<String> = method.params.iter().map(|p| p.name.clone()).collect();
    let is_static = (method.flags & 0x0010) != 0;

    let resolver = MetadataResolver::new(assembly);

    // Use control flow analyzer for structured output (if/else, loops, try/catch).
    let analyzer = ControlFlowAnalyzer::new(
        &resolver,
        &parsed_body.instructions,
        &parsed_body.exception_handlers,
        parsed_body.locals.clone(),
        params,
        is_static,
    );
    let statements = analyzer.analyze();
    let statements = patterns::apply_patterns(statements, enclosing_type, lambda_map);

    Ok(emit_statements(&statements, indent))
}

/// Emit constructor statements, skipping assignments to fields that have been hoisted to initializers.
pub(in crate::decompiler) fn emit_constructor_body_filtered(
    stmts: &[Statement],
    hoisted_fields: &HashSet<String>,
    indent: usize,
) -> String {
    use crate::decompiler::ast::Expr;
    let filtered: Vec<Statement> = stmts
        .iter()
        .filter(|stmt| {
            if let Statement::Assign(Expr::Field(obj, name), _) = stmt {
                if matches!(obj.as_ref(), Expr::This) && hoisted_fields.contains(name.as_str()) {
                    return false;
                }
            }
            true
        })
        .cloned()
        .collect();
    emit_statements(&filtered, indent)
}
