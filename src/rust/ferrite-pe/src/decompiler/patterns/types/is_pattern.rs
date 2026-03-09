//! Pattern matching extensions — `is` declaration patterns.
//!
//! Detects `var x = expr as Type; if (x != null) { ... }` and rewrites to
//! `if (expr is Type x) { ... }`.

use crate::decompiler::ast::*;
use crate::decompiler::emit::emit_expr;

/// Detect `as` + null-check patterns and rewrite to `is` declaration patterns.
pub fn detect_is_pattern(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result = Vec::with_capacity(stmts.len());
    let mut i = 0;
    let stmts_vec: Vec<Statement> = stmts.into_iter().map(recurse_is).collect();
    while i < stmts_vec.len() {
        if let Some((rewritten, consumed)) = try_detect_is_at(&stmts_vec, i) {
            result.push(rewritten);
            i += consumed;
        } else {
            result.push(stmts_vec[i].clone());
            i += 1;
        }
    }
    result
}

/// Try to detect `LocalDecl(ty, name, AsInst(expr, cast_ty))` followed by
/// `If(name != null, then, else)` and rewrite to `If(expr is Type name, ...)`.
fn try_detect_is_at(stmts: &[Statement], idx: usize) -> Option<(Statement, usize)> {
    let decl = stmts.get(idx)?;
    let (var_name, inner_expr, cast_ty) = match decl {
        Statement::LocalDecl(_, name, Some(Expr::AsInst(expr, ty))) => {
            (name.clone(), expr, ty.clone())
        }
        _ => return None,
    };

    let if_stmt = stmts.get(idx + 1)?;
    let (cond, then_block, else_block) = match if_stmt {
        Statement::If(c, tb, eb) => (c, tb, eb),
        _ => return None,
    };

    // Check that the condition is `var_name != null`
    let checked_var = super::super::helpers::extract_null_check_ne(cond)?;
    if checked_var != var_name {
        return None;
    }

    let is_expr = Expr::Raw(format!(
        "{} is {} {}",
        emit_expr(inner_expr),
        cast_ty,
        var_name
    ));
    let rewritten = Statement::If(
        is_expr,
        detect_is_pattern(then_block.clone()),
        else_block.as_ref().map(|b| detect_is_pattern(b.clone())),
    );
    Some((rewritten, 2))
}

/// Recurse into all statement variants.
fn recurse_is(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(c, tb, eb) => {
            Statement::If(c, detect_is_pattern(tb), eb.map(detect_is_pattern))
        }
        Statement::While(c, b) => Statement::While(c, detect_is_pattern(b)),
        Statement::DoWhile(b, c) => Statement::DoWhile(detect_is_pattern(b), c),
        Statement::For(i, c, u, b) => Statement::For(i, c, u, detect_is_pattern(b)),
        Statement::ForEach(t, n, col, b) => Statement::ForEach(t, n, col, detect_is_pattern(b)),
        Statement::Try(tb, catches, fb) => {
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_is_pattern(c.body),
                    ..c
                })
                .collect();
            Statement::Try(detect_is_pattern(tb), catches, fb.map(detect_is_pattern))
        }
        Statement::Using(d, b) => Statement::Using(d, detect_is_pattern(b)),
        Statement::Lock(e, b) => Statement::Lock(e, detect_is_pattern(b)),
        Statement::Checked(b) => Statement::Checked(detect_is_pattern(b)),
        Statement::Unchecked(b) => Statement::Unchecked(detect_is_pattern(b)),
        Statement::Fixed(t, n, e, b) => Statement::Fixed(t, n, e, detect_is_pattern(b)),
        Statement::Switch(e, cases, def) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_is_pattern(b)))
                .collect();
            Statement::Switch(e, cases, def.map(detect_is_pattern))
        }
        other => other,
    }
}
