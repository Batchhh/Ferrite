//! C# pattern recognition — post-processing pass.

pub mod accessors;
pub mod array;
pub mod booleans;
pub mod cleanup;
pub mod compiler;
pub mod conditions;
pub mod delegates;
pub mod lambdas;
pub mod loops_for;
pub mod loops_foreach;
pub mod null_coalescing;
pub mod using;

use crate::decompiler::ast::Statement;
use std::collections::HashMap;

/// Map from lambda method name → (params [(type, name)], body statements).
pub type LambdaMap = HashMap<String, (Vec<(String, String)>, Vec<Statement>)>;

/// Apply all pattern transformations to a list of statements.
pub fn apply_patterns(
    statements: Vec<Statement>,
    enclosing_type: &str,
    lambda_map: &LambdaMap,
) -> Vec<Statement> {
    let mut stmts = statements;
    stmts = array::detect_array_initializers(stmts);
    stmts = accessors::rewrite_property_accessors(stmts);
    stmts = conditions::simplify_conditions(stmts);
    stmts = loops_foreach::detect_foreach(stmts);
    stmts = using::detect_using(stmts);
    stmts = null_coalescing::detect_null_coalescing(stmts);
    stmts = compiler::rewrite_ctor_calls(stmts);
    stmts = compiler::simplify_compiler_generated(stmts);
    stmts = lambdas::inline_lambdas(stmts, lambda_map);
    stmts = delegates::propagate_delegate_assignments(stmts);
    stmts = compiler::simplify_self_references(stmts, enclosing_type);
    stmts = loops_for::detect_for_loops(stmts);
    stmts = booleans::coerce_booleans(stmts);
    stmts = cleanup::clean_up(stmts);
    stmts
}
