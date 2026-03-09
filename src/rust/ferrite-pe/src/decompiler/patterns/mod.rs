//! C# pattern recognition — post-processing pass.

pub mod cleanup;
pub mod compiler;
pub mod helpers;

pub mod control_flow;
pub mod expressions;
pub mod null;
pub mod resources;
pub mod types;

pub use control_flow::*;
pub use expressions::*;
pub use null::*;
pub use resources::*;
pub use types::*;

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
    stmts = string_interpolation::detect_string_interpolation(stmts);
    stmts = range_index::detect_range_index(stmts);
    stmts = accessors::rewrite_property_accessors(stmts);
    stmts = conditions::simplify_conditions(stmts);
    stmts = lock::detect_lock(stmts);
    stmts = loops_foreach::detect_foreach(stmts);
    stmts = using::detect_using(stmts);
    stmts = using_decl::detect_using_decl(stmts);
    stmts = null_coalescing::detect_null_coalescing(stmts);
    stmts = null_conditional::detect_null_conditional(stmts);
    stmts = compiler::rewrite_ctor_calls(stmts);
    stmts = compiler::simplify_compiler_generated(stmts);
    stmts = lambdas::inline_lambdas(stmts, lambda_map);
    stmts = delegates::propagate_delegate_assignments(stmts);
    stmts = compiler::simplify_self_references(stmts, enclosing_type);
    stmts = tuples::detect_tuples(stmts);
    stmts = loops_for::detect_for_loops(stmts);
    stmts = checked::detect_checked(stmts);
    stmts = booleans::coerce_booleans(stmts);
    stmts = switch_expr::detect_switch_expr(stmts);
    stmts = is_pattern::detect_is_pattern(stmts);
    stmts = fixed::detect_fixed(stmts);
    stmts = async_await::detect_async_await(stmts);
    stmts = yield_return::detect_yield_return(stmts);
    stmts = cleanup::clean_up(stmts);
    stmts
}
