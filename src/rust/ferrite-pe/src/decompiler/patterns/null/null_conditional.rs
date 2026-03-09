//! Null-conditional (`?.`) pattern detection.
//!
//! Rewrites compiler-generated `if (x != null) x.Foo` patterns into
//! `x?.Foo` (NullConditionalAccess / NullConditionalCall).

use super::super::helpers::*;
use crate::decompiler::ast::*;

pub fn detect_null_conditional(stmts: Vec<Statement>) -> Vec<Statement> {
    stmts
        .into_iter()
        .map(detect_null_conditional_stmt)
        .collect()
}

fn detect_null_conditional_stmt(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(cond, then_block, Some(else_block)) => {
            if let Some(s) = try_property_or_call_with_else(&cond, &then_block, &else_block) {
                return s;
            }
            Statement::If(
                cond,
                detect_null_conditional(then_block),
                Some(detect_null_conditional(else_block)),
            )
        }
        Statement::If(cond, then_block, None) => {
            if let Some(s) = try_void_call(&cond, &then_block) {
                return s;
            }
            Statement::If(cond, detect_null_conditional(then_block), None)
        }
        other => recurse_null_conditional(other),
    }
}

fn recurse_null_conditional(stmt: Statement) -> Statement {
    match stmt {
        Statement::While(c, b) => Statement::While(c, detect_null_conditional(b)),
        Statement::DoWhile(b, c) => Statement::DoWhile(detect_null_conditional(b), c),
        Statement::For(i, c, u, b) => Statement::For(i, c, u, detect_null_conditional(b)),
        Statement::ForEach(t, n, col, b) => {
            Statement::ForEach(t, n, col, detect_null_conditional(b))
        }
        Statement::Try(tb, catches, fb) => {
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_null_conditional(c.body),
                    ..c
                })
                .collect();
            Statement::Try(
                detect_null_conditional(tb),
                catches,
                fb.map(detect_null_conditional),
            )
        }
        Statement::Using(d, b) => Statement::Using(d, detect_null_conditional(b)),
        Statement::Lock(e, b) => Statement::Lock(e, detect_null_conditional(b)),
        Statement::Checked(b) => Statement::Checked(detect_null_conditional(b)),
        Statement::Unchecked(b) => Statement::Unchecked(detect_null_conditional(b)),
        Statement::Fixed(t, n, e, b) => Statement::Fixed(t, n, e, detect_null_conditional(b)),
        Statement::Switch(e, cases, def) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_null_conditional(b)))
                .collect();
            Statement::Switch(e, cases, def.map(detect_null_conditional))
        }
        other => other,
    }
}

/// Pattern 1 & 3: `if (x != null) { r = x.Prop/x.M(); } else { r = null; }`
fn try_property_or_call_with_else(
    cond: &Expr,
    then_block: &[Statement],
    else_block: &[Statement],
) -> Option<Statement> {
    let checked_var = extract_null_check_ne(cond)?;
    if then_block.len() != 1 || else_block.len() != 1 {
        return None;
    }
    let (then_target, then_value) = extract_assignment(&then_block[0])?;
    let (else_target, else_value) = extract_assignment(&else_block[0])?;
    if !exprs_equal(&then_target, &else_target) || !matches!(else_value, Expr::Null) {
        return None;
    }
    match then_value {
        Expr::Field(ref obj, ref name) if expr_references_var(obj, &checked_var) => {
            Some(Statement::Assign(
                then_target,
                Expr::NullConditionalAccess(obj.clone(), name.clone()),
            ))
        }
        Expr::Call(Some(ref obj), ref name, ref args) if expr_references_var(obj, &checked_var) => {
            Some(Statement::Assign(
                then_target,
                Expr::NullConditionalCall(obj.clone(), name.clone(), args.clone()),
            ))
        }
        _ => None,
    }
}

/// Pattern 2: `if (x != null) { x.Method(args); }` (no else, void call)
fn try_void_call(cond: &Expr, then_block: &[Statement]) -> Option<Statement> {
    let checked_var = extract_null_check_ne(cond)?;
    if then_block.len() != 1 {
        return None;
    }
    match &then_block[0] {
        Statement::Expr(Expr::Call(Some(obj), name, args))
            if expr_references_var(obj, &checked_var) =>
        {
            Some(Statement::Expr(Expr::NullConditionalCall(
                obj.clone(),
                name.clone(),
                args.clone(),
            )))
        }
        _ => None,
    }
}
