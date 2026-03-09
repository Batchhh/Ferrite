//! Switch expression pattern detection.
//!
//! Detects switch statements where every case assigns to the same variable
//! and rewrites them as switch expressions:
//! `target = expr switch { val => result, ... };`

use super::super::helpers::exprs_equal;
use crate::decompiler::ast::*;

/// Detect switch expressions and rewrite them.
pub fn detect_switch_expr(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result = Vec::with_capacity(stmts.len());
    for stmt in stmts {
        let stmt = recurse_switch_expr(stmt);
        if let Some(rewritten) = try_rewrite_switch_expr(stmt.clone()) {
            result.push(rewritten);
        } else {
            result.push(stmt);
        }
    }
    result
}

/// Try to rewrite a Switch statement into an Assign with a SwitchExpr.
fn try_rewrite_switch_expr(stmt: Statement) -> Option<Statement> {
    let (switch_expr, cases, default) = match stmt {
        Statement::Switch(ref e, ref c, ref d) => (e, c, d),
        _ => return None,
    };

    if cases.is_empty() {
        return None;
    }

    // Extract assignment target from first case to establish the pattern
    let first_target = extract_case_target(&cases[0].1)?;

    // Verify all cases match: exactly Assign(target, value) + Break
    let mut arms = Vec::new();
    for (labels, body) in cases {
        let (target, value) = extract_case_assignment(body)?;
        if !exprs_equal(&target, &first_target) {
            return None;
        }
        // Each label gets paired with the same result value
        for label in labels {
            arms.push((label.clone(), value.clone()));
        }
    }

    // Check default block if present
    let default_val = if let Some(def_body) = default {
        let (target, value) = extract_case_assignment(def_body)?;
        if !exprs_equal(&target, &first_target) {
            return None;
        }
        Some(Box::new(value))
    } else {
        None
    };

    Some(Statement::Assign(
        first_target,
        Expr::SwitchExpr(Box::new(switch_expr.clone()), arms, default_val),
    ))
}

/// Extract the assignment target from a case body (Assign + Break).
fn extract_case_target(body: &[Statement]) -> Option<Expr> {
    if body.len() != 2 {
        return None;
    }
    if !matches!(body[1], Statement::Break) {
        return None;
    }
    match &body[0] {
        Statement::Assign(target, _) => Some(target.clone()),
        _ => None,
    }
}

/// Extract (target, value) from a case body that is Assign + Break.
fn extract_case_assignment(body: &[Statement]) -> Option<(Expr, Expr)> {
    if body.len() != 2 {
        return None;
    }
    if !matches!(body[1], Statement::Break) {
        return None;
    }
    match &body[0] {
        Statement::Assign(target, value) => Some((target.clone(), value.clone())),
        _ => None,
    }
}

/// Recurse into all statement variants.
fn recurse_switch_expr(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(c, tb, eb) => {
            Statement::If(c, detect_switch_expr(tb), eb.map(detect_switch_expr))
        }
        Statement::While(c, b) => Statement::While(c, detect_switch_expr(b)),
        Statement::DoWhile(b, c) => Statement::DoWhile(detect_switch_expr(b), c),
        Statement::For(i, c, u, b) => Statement::For(i, c, u, detect_switch_expr(b)),
        Statement::ForEach(t, n, col, b) => Statement::ForEach(t, n, col, detect_switch_expr(b)),
        Statement::Try(tb, catches, fb) => {
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_switch_expr(c.body),
                    ..c
                })
                .collect();
            Statement::Try(detect_switch_expr(tb), catches, fb.map(detect_switch_expr))
        }
        Statement::Using(d, b) => Statement::Using(d, detect_switch_expr(b)),
        Statement::Lock(e, b) => Statement::Lock(e, detect_switch_expr(b)),
        Statement::Checked(b) => Statement::Checked(detect_switch_expr(b)),
        Statement::Unchecked(b) => Statement::Unchecked(detect_switch_expr(b)),
        Statement::Fixed(t, n, e, b) => Statement::Fixed(t, n, e, detect_switch_expr(b)),
        Statement::Switch(e, cases, def) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_switch_expr(b)))
                .collect();
            Statement::Switch(e, cases, def.map(detect_switch_expr))
        }
        other => other,
    }
}
