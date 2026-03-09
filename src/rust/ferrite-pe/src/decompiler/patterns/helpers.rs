//! Shared helper functions for pattern recognition.

use crate::decompiler::ast::*;

/// Extract the variable name from a local or argument expression.
pub(super) fn expr_var_name(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Local(_, name) => Some(name.clone()),
        Expr::Arg(_, name) => Some(name.clone()),
        _ => None,
    }
}

/// Check if a finally block calls Dispose (directly or inside an if).
pub(super) fn finally_calls_dispose(finally: &[Statement]) -> bool {
    for stmt in finally {
        match stmt {
            Statement::Expr(Expr::Call(_, name, _)) if name == "Dispose" => return true,
            Statement::If(_, then_block, _) => {
                if finally_calls_dispose(then_block) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Extract `var_name` from `var != null` check.
pub(super) fn extract_null_check_ne(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Binary(left, BinOp::Ne, right) if matches!(right.as_ref(), Expr::Null) => {
            expr_var_name(left)
        }
        Expr::Binary(left, BinOp::Ne, right) if matches!(left.as_ref(), Expr::Null) => {
            expr_var_name(right)
        }
        _ => None,
    }
}

/// Extract `var_name` from `var == null` check.
pub(super) fn extract_null_check_eq(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Binary(left, BinOp::Eq, right) if matches!(right.as_ref(), Expr::Null) => {
            expr_var_name(left)
        }
        Expr::Binary(left, BinOp::Eq, right) if matches!(left.as_ref(), Expr::Null) => {
            expr_var_name(right)
        }
        _ => None,
    }
}

/// Extract (target, value) from an assignment statement.
pub(super) fn extract_assignment(stmt: &Statement) -> Option<(Expr, Expr)> {
    match stmt {
        Statement::Assign(target, value) => Some((target.clone(), value.clone())),
        _ => None,
    }
}

/// Check structural equality of two expressions (by position, not name).
pub(super) fn exprs_equal(a: &Expr, b: &Expr) -> bool {
    match (a, b) {
        (Expr::Local(i1, _), Expr::Local(i2, _)) => i1 == i2,
        (Expr::Arg(i1, _), Expr::Arg(i2, _)) => i1 == i2,
        (Expr::Field(o1, n1), Expr::Field(o2, n2)) => n1 == n2 && exprs_equal(o1, o2),
        (Expr::StaticField(t1, n1), Expr::StaticField(t2, n2)) => t1 == t2 && n1 == n2,
        (Expr::This, Expr::This) => true,
        _ => false,
    }
}

/// Check if an expression references a variable by name.
pub(super) fn expr_references_var(expr: &Expr, var_name: &str) -> bool {
    match expr {
        Expr::Local(_, name) | Expr::Arg(_, name) => name == var_name,
        _ => false,
    }
}
