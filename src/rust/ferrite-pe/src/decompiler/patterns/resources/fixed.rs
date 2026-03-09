//! Fixed statement pattern detection.
//!
//! Detects pointer pinning patterns and rewrites them to `fixed` statements:
//! `LocalDecl("byte*", "ptr", AddressOf(expr))` ... `Assign(ptr, null)` →
//! `Fixed("byte*", "ptr", expr, body)`.

use crate::decompiler::ast::*;

/// Detect fixed statement patterns and rewrite them.
pub fn detect_fixed(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result = Vec::with_capacity(stmts.len());
    let mut i = 0;
    let stmts_vec: Vec<Statement> = stmts.into_iter().map(recurse_fixed).collect();
    while i < stmts_vec.len() {
        if let Some((fixed_stmt, consumed)) = try_detect_fixed_at(&stmts_vec, i) {
            result.push(fixed_stmt);
            i += consumed;
        } else {
            result.push(stmts_vec[i].clone());
            i += 1;
        }
    }
    result
}

/// Try to detect a fixed pattern starting at `idx`.
fn try_detect_fixed_at(stmts: &[Statement], idx: usize) -> Option<(Statement, usize)> {
    let decl = stmts.get(idx)?;
    let (ty, name, inner) = match decl {
        Statement::LocalDecl(ty, name, Some(Expr::AddressOf(inner))) if ty.contains('*') => {
            (ty.clone(), name.clone(), inner.as_ref().clone())
        }
        _ => return None,
    };

    // Find the null assignment that ends the fixed block
    let end_idx = stmts
        .iter()
        .enumerate()
        .skip(idx + 1)
        .find(|(_, s)| is_null_assign(s, &name))
        .map(|(j, _)| j)?;

    let body: Vec<Statement> = stmts[idx + 1..end_idx].to_vec();
    let body = detect_fixed(body);
    let fixed = Statement::Fixed(ty, name, inner, body);
    Some((fixed, end_idx - idx + 1))
}

/// Check if a statement is `Assign(Local(_, name), Null)` or
/// `Assign(Local(_, name), Int(0))`.
fn is_null_assign(stmt: &Statement, var_name: &str) -> bool {
    match stmt {
        Statement::Assign(Expr::Local(_, name), Expr::Null) if name == var_name => true,
        Statement::Assign(Expr::Local(_, name), Expr::Int(0)) if name == var_name => true,
        _ => false,
    }
}

/// Recurse into all statement variants.
fn recurse_fixed(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(c, tb, eb) => Statement::If(c, detect_fixed(tb), eb.map(detect_fixed)),
        Statement::While(c, b) => Statement::While(c, detect_fixed(b)),
        Statement::DoWhile(b, c) => Statement::DoWhile(detect_fixed(b), c),
        Statement::For(i, c, u, b) => Statement::For(i, c, u, detect_fixed(b)),
        Statement::ForEach(t, n, col, b) => Statement::ForEach(t, n, col, detect_fixed(b)),
        Statement::Try(tb, catches, fb) => {
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_fixed(c.body),
                    ..c
                })
                .collect();
            Statement::Try(detect_fixed(tb), catches, fb.map(detect_fixed))
        }
        Statement::Using(d, b) => Statement::Using(d, detect_fixed(b)),
        Statement::Lock(e, b) => Statement::Lock(e, detect_fixed(b)),
        Statement::Checked(b) => Statement::Checked(detect_fixed(b)),
        Statement::Unchecked(b) => Statement::Unchecked(detect_fixed(b)),
        Statement::Fixed(t, n, e, b) => Statement::Fixed(t, n, e, detect_fixed(b)),
        Statement::Switch(e, cases, def) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_fixed(b)))
                .collect();
            Statement::Switch(e, cases, def.map(detect_fixed))
        }
        other => other,
    }
}
