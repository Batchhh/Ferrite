use crate::assembly::{Assembly, MethodDef, TypeDef};
use crate::decompiler::ast::{Expr, Statement};
use crate::decompiler::control_flow::ControlFlowAnalyzer;
use crate::decompiler::emit::emit_expr;
use crate::decompiler::patterns;
use crate::decompiler::patterns::LambdaMap;
use crate::decompiler::resolver::MetadataResolver;
use std::collections::HashMap;

/// Build a map from lambda method names to their decompiled bodies.
/// Scans compiler-generated nested types (e.g. `<>c`) for methods that are
/// lambda body implementations.
pub(in crate::decompiler) fn build_lambda_map(td: &TypeDef, assembly: &Assembly) -> LambdaMap {
    let mut map = LambdaMap::new();
    let empty_lm = LambdaMap::new();

    for nested in &td.nested_types {
        if !is_compiler_generated_type(&nested.name) {
            continue;
        }
        for method in &nested.methods {
            // Skip constructors and static constructors
            if &*method.name == ".ctor" || &*method.name == ".cctor" {
                continue;
            }

            // Extract params (skip `this` — instance methods on the <>c class
            // receive the singleton as arg0, which we don't want in the lambda)
            let is_static = (method.flags & 0x0010) != 0;
            let params: Vec<(String, String)> = method
                .params
                .iter()
                .map(|p| (p.type_name.clone(), p.name.clone()))
                .collect();

            // Decompile the method body (using an empty lambda map to avoid recursion)
            if let Some(parsed_body) = &method.parsed_body {
                let param_names: Vec<String> =
                    method.params.iter().map(|p| p.name.clone()).collect();
                let resolver = MetadataResolver::new(assembly);
                let analyzer = ControlFlowAnalyzer::new(
                    &resolver,
                    &parsed_body.instructions,
                    &parsed_body.exception_handlers,
                    parsed_body.locals.clone(),
                    param_names,
                    is_static,
                );
                let stmts = analyzer.analyze();
                // Apply basic patterns (property rewriting etc.) but with empty lambda map
                let stmts = patterns::apply_patterns(stmts, &nested.name, &empty_lm);
                map.insert(method.name.to_string(), (params, stmts));
            }
        }
    }

    map
}

/// Build the AST statements for a method body (without emitting to string).
pub(in crate::decompiler) fn build_method_statements(
    method: &MethodDef,
    assembly: &Assembly,
    enclosing_type: &str,
    lambda_map: &LambdaMap,
) -> Option<Vec<Statement>> {
    let parsed_body = method.parsed_body.as_ref()?;
    let params: Vec<String> = method.params.iter().map(|p| p.name.clone()).collect();
    let is_static = (method.flags & 0x0010) != 0;
    let resolver = MetadataResolver::new(assembly);
    let analyzer = ControlFlowAnalyzer::new(
        &resolver,
        &parsed_body.instructions,
        &parsed_body.exception_handlers,
        parsed_body.locals.clone(),
        params,
        is_static,
    );
    let statements = analyzer.analyze();
    Some(patterns::apply_patterns(
        statements,
        enclosing_type,
        lambda_map,
    ))
}

/// Returns true for compiler-generated nested type names that should be hidden.
/// Examples: `<>c`, `<PrivateImplementationDetails>`, `<DoStuff>d__5`,
/// `<>c__DisplayClass0_0`.
pub(in crate::decompiler) fn is_compiler_generated_type(name: &str) -> bool {
    // Names starting with "<>" are always compiler-generated (e.g. `<>c`)
    if name.starts_with("<>") {
        return true;
    }
    // Names starting with "<" followed by a non-alphabetic char (e.g. `<1>`)
    if name.starts_with('<') {
        if let Some(second) = name.chars().nth(1) {
            if !second.is_alphabetic() {
                return true;
            }
        }
    }
    // Closure capture / display classes
    if name.contains("__DisplayClass") {
        return true;
    }
    false
}

/// Check if an expression is a "simple" initializer suitable for hoisting
/// (NewObj, literal, null, default, etc.).
pub(in crate::decompiler) fn is_simple_initializer(expr: &Expr) -> bool {
    match expr {
        Expr::Null | Expr::Bool(_) | Expr::Int(_) | Expr::Float(_) | Expr::String(_) => true,
        Expr::NewObj(_, args) => args.iter().all(is_simple_initializer),
        Expr::Default(_) | Expr::Typeof(_) => true,
        Expr::ArrayNew(_, size) => is_simple_initializer(size),
        _ => false,
    }
}

/// Extract leading field initializations from a constructor's statement list.
/// Returns a vec of (field_name, initializer_expr_string) for leading
/// `Assign(Field(This, name), expr)` statements where expr is simple.
pub(in crate::decompiler) fn extract_leading_field_inits(
    stmts: &[Statement],
) -> Vec<(String, String)> {
    let mut result = Vec::new();
    for stmt in stmts {
        match stmt {
            Statement::Assign(Expr::Field(obj, name), value)
                if matches!(obj.as_ref(), Expr::This) && is_simple_initializer(value) =>
            {
                result.push((name.clone(), emit_expr(value)));
            }
            // base() call (rewritten ctor call) — skip over it, it's not a field init
            // but it's expected at the start of a constructor
            Statement::Expr(Expr::Call(Some(obj), name, _))
                if matches!(obj.as_ref(), Expr::This) && name == "base" => {}
            // Any other statement — stop looking for field inits
            _ => break,
        }
    }
    result
}

/// Analyze all instance constructors of a type and find field initializations
/// that are common to ALL constructors (same field, same initializer expression).
/// Returns a map of field_name → initializer_expression_string.
pub(in crate::decompiler) fn extract_field_initializers(
    td: &TypeDef,
    assembly: &Assembly,
) -> HashMap<String, String> {
    let instance_ctors: Vec<&MethodDef> = td
        .methods
        .iter()
        .filter(|m| &*m.name == ".ctor" && m.rva != 0)
        .collect();

    if instance_ctors.is_empty() {
        return HashMap::new();
    }

    // For each constructor, get leading field inits
    let mut per_ctor_inits: Vec<HashMap<String, String>> = Vec::new();
    for ctor in &instance_ctors {
        let empty_lm = LambdaMap::new();
        if let Some(stmts) = build_method_statements(ctor, assembly, &td.name, &empty_lm) {
            let inits: HashMap<String, String> =
                extract_leading_field_inits(&stmts).into_iter().collect();
            per_ctor_inits.push(inits);
        } else {
            // If we can't decompile any ctor, bail out
            return HashMap::new();
        }
    }

    if per_ctor_inits.is_empty() {
        return HashMap::new();
    }

    // Find fields that ALL constructors initialize with the same value
    let first = &per_ctor_inits[0];
    let mut common = HashMap::new();
    for (field_name, init_expr) in first {
        let all_same = per_ctor_inits[1..]
            .iter()
            .all(|inits| inits.get(field_name) == Some(init_expr));
        if all_same {
            // Verify this field exists on the type and is not a backing field
            if td
                .fields
                .iter()
                .any(|f| *f.name == **field_name && !is_backing_field(&f.name))
            {
                common.insert(field_name.clone(), init_expr.clone());
            }
        }
    }

    common
}

pub(in crate::decompiler) fn is_backing_field(name: &str) -> bool {
    name.starts_with('<') && name.contains(">k__BackingField")
}
