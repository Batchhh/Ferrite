//! Checked arithmetic pattern detection.
//!
//! Groups contiguous statements using checked arithmetic ops
//! (`AddChecked`, `SubChecked`, `MulChecked`) into `checked { }` blocks.

use crate::decompiler::ast::*;

pub fn detect_checked(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result = Vec::new();
    let mut checked_group: Vec<Statement> = Vec::new();

    for stmt in stmts {
        let stmt = recurse_checked(stmt);
        if stmt_has_checked_ops(&stmt) {
            checked_group.push(stmt);
        } else {
            if !checked_group.is_empty() {
                result.push(Statement::Checked(std::mem::take(&mut checked_group)));
            }
            result.push(stmt);
        }
    }
    if !checked_group.is_empty() {
        result.push(Statement::Checked(checked_group));
    }
    result
}

fn recurse_checked(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(c, tb, eb) => Statement::If(c, detect_checked(tb), eb.map(detect_checked)),
        Statement::While(c, b) => Statement::While(c, detect_checked(b)),
        Statement::DoWhile(b, c) => Statement::DoWhile(detect_checked(b), c),
        Statement::For(i, c, u, b) => Statement::For(i, c, u, detect_checked(b)),
        Statement::ForEach(t, n, col, b) => Statement::ForEach(t, n, col, detect_checked(b)),
        Statement::Try(tb, catches, fb) => {
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_checked(c.body),
                    ..c
                })
                .collect();
            Statement::Try(detect_checked(tb), catches, fb.map(detect_checked))
        }
        Statement::Using(d, b) => Statement::Using(d, detect_checked(b)),
        Statement::Lock(e, b) => Statement::Lock(e, detect_checked(b)),
        Statement::Checked(b) => Statement::Checked(detect_checked(b)),
        Statement::Unchecked(b) => Statement::Unchecked(detect_checked(b)),
        Statement::Fixed(t, n, e, b) => Statement::Fixed(t, n, e, detect_checked(b)),
        Statement::Switch(e, cases, def) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_checked(b)))
                .collect();
            Statement::Switch(e, cases, def.map(detect_checked))
        }
        other => other,
    }
}

/// Check if a statement contains checked arithmetic ops.
fn stmt_has_checked_ops(stmt: &Statement) -> bool {
    match stmt {
        Statement::Assign(_, value) => expr_has_checked_ops(value),
        Statement::LocalDecl(_, _, Some(init)) => expr_has_checked_ops(init),
        Statement::Return(Some(e)) => expr_has_checked_ops(e),
        Statement::Expr(e) => expr_has_checked_ops(e),
        _ => false,
    }
}

/// Recursively check if any sub-expression uses checked BinOps.
fn expr_has_checked_ops(expr: &Expr) -> bool {
    match expr {
        Expr::Binary(l, op, r) => {
            matches!(
                op,
                BinOp::AddChecked | BinOp::SubChecked | BinOp::MulChecked
            ) || expr_has_checked_ops(l)
                || expr_has_checked_ops(r)
        }
        Expr::Unary(_, e) => expr_has_checked_ops(e),
        Expr::Call(obj, _, args) => {
            obj.as_ref().is_some_and(|o| expr_has_checked_ops(o))
                || args.iter().any(expr_has_checked_ops)
        }
        Expr::StaticCall(_, _, args) | Expr::NewObj(_, args) => {
            args.iter().any(expr_has_checked_ops)
        }
        Expr::Field(o, _) => expr_has_checked_ops(o),
        Expr::Cast(_, e) | Expr::Box(_, e) | Expr::Unbox(_, e) | Expr::AddressOf(e) => {
            expr_has_checked_ops(e)
        }
        Expr::ArrayElement(a, i) => expr_has_checked_ops(a) || expr_has_checked_ops(i),
        Expr::Ternary(c, t, e) => {
            expr_has_checked_ops(c) || expr_has_checked_ops(t) || expr_has_checked_ops(e)
        }
        _ => false,
    }
}
