//! Yield return/break pattern detection.
//!
//! Detects compiler-generated iterator patterns:
//! - `this.<>2__current = expr; ... return true;` → `yield return expr;`
//! - `return false;` → `yield break;`

use crate::decompiler::ast::*;

/// Detect yield return/break patterns and rewrite them.
pub fn detect_yield_return(stmts: Vec<Statement>) -> Vec<Statement> {
    let stmts_vec: Vec<Statement> = stmts.into_iter().map(recurse_yield).collect();
    rewrite_yield_sequence(stmts_vec)
}

/// Scan for `__current` field assignments followed by `return true`.
fn rewrite_yield_sequence(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result = Vec::with_capacity(stmts.len());
    let mut has_yield = false;
    let mut i = 0;
    while i < stmts.len() {
        if let Some((yield_stmt, consumed)) = try_yield_return(&stmts, i) {
            result.push(yield_stmt);
            has_yield = true;
            i += consumed;
        } else if has_yield && is_return_false(&stmts[i]) {
            result.push(Statement::YieldBreak);
            i += 1;
        } else {
            result.push(stmts[i].clone());
            i += 1;
        }
    }
    result
}

/// Try to detect `Assign(Field(This, *__current), value)` followed by
/// `Return(Some(Bool(true)))`.
fn try_yield_return(stmts: &[Statement], idx: usize) -> Option<(Statement, usize)> {
    let assign = stmts.get(idx)?;
    let value = match assign {
        Statement::Assign(Expr::Field(obj, field), val)
            if matches!(obj.as_ref(), Expr::This) && field.contains("__current") =>
        {
            val.clone()
        }
        _ => return None,
    };

    // Look for `return true` after the assignment, skipping state assignments
    for (j, stmt) in stmts.iter().enumerate().skip(idx + 1) {
        if is_return_true(stmt) {
            return Some((Statement::YieldReturn(value), j - idx + 1));
        }
        // Skip state field assignments (e.g. `this.<>1__state = N`)
        if is_state_field_assign(stmt) {
            continue;
        }
        break;
    }
    None
}

fn is_return_true(stmt: &Statement) -> bool {
    matches!(stmt, Statement::Return(Some(Expr::Bool(true))))
}

fn is_return_false(stmt: &Statement) -> bool {
    matches!(stmt, Statement::Return(Some(Expr::Bool(false))))
}

fn is_state_field_assign(stmt: &Statement) -> bool {
    matches!(
        stmt,
        Statement::Assign(Expr::Field(obj, field), _)
            if matches!(obj.as_ref(), Expr::This) && field.contains("__state")
    )
}

/// Recurse into all statement variants.
fn recurse_yield(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(c, tb, eb) => {
            Statement::If(c, detect_yield_return(tb), eb.map(detect_yield_return))
        }
        Statement::While(c, b) => Statement::While(c, detect_yield_return(b)),
        Statement::DoWhile(b, c) => Statement::DoWhile(detect_yield_return(b), c),
        Statement::For(i, c, u, b) => Statement::For(i, c, u, detect_yield_return(b)),
        Statement::ForEach(t, n, col, b) => Statement::ForEach(t, n, col, detect_yield_return(b)),
        Statement::Try(tb, catches, fb) => {
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_yield_return(c.body),
                    ..c
                })
                .collect();
            Statement::Try(
                detect_yield_return(tb),
                catches,
                fb.map(detect_yield_return),
            )
        }
        Statement::Using(d, b) => Statement::Using(d, detect_yield_return(b)),
        Statement::Lock(e, b) => Statement::Lock(e, detect_yield_return(b)),
        Statement::Checked(b) => Statement::Checked(detect_yield_return(b)),
        Statement::Unchecked(b) => Statement::Unchecked(detect_yield_return(b)),
        Statement::Fixed(t, n, e, b) => Statement::Fixed(t, n, e, detect_yield_return(b)),
        Statement::Switch(e, cases, def) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_yield_return(b)))
                .collect();
            Statement::Switch(e, cases, def.map(detect_yield_return))
        }
        other => other,
    }
}
