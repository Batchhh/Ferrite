use super::super::helpers::*;
use crate::decompiler::ast::*;

pub fn detect_null_coalescing(stmts: Vec<Statement>) -> Vec<Statement> {
    stmts.into_iter().map(detect_null_coalescing_stmt).collect()
}

fn detect_null_coalescing_stmt(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(cond, then_block, Some(else_block)) => {
            if let Some(nc) = try_null_coalescing_if(&cond, &then_block, &else_block) {
                return nc;
            }
            if let Some(nc) = try_null_coalescing_if_reversed(&cond, &then_block, &else_block) {
                return nc;
            }
            Statement::If(
                cond,
                detect_null_coalescing(then_block),
                Some(detect_null_coalescing(else_block)),
            )
        }
        Statement::If(cond, then_block, None) => {
            Statement::If(cond, detect_null_coalescing(then_block), None)
        }
        Statement::While(cond, body) => Statement::While(cond, detect_null_coalescing(body)),
        Statement::DoWhile(body, cond) => Statement::DoWhile(detect_null_coalescing(body), cond),
        Statement::For(init, cond, update, body) => {
            Statement::For(init, cond, update, detect_null_coalescing(body))
        }
        Statement::ForEach(ty, name, coll, body) => {
            Statement::ForEach(ty, name, coll, detect_null_coalescing(body))
        }
        Statement::Try(try_body, catches, finally_block) => {
            let try_body = detect_null_coalescing(try_body);
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_null_coalescing(c.body),
                    ..c
                })
                .collect();
            let finally_block = finally_block.map(detect_null_coalescing);
            Statement::Try(try_body, catches, finally_block)
        }
        Statement::Using(decl, body) => Statement::Using(decl, detect_null_coalescing(body)),
        Statement::Switch(expr, cases, default) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_null_coalescing(b)))
                .collect();
            Statement::Switch(expr, cases, default.map(detect_null_coalescing))
        }
        other => other,
    }
}

/// if (x != null) { target = x; } else { target = y; } -> target = x ?? y
fn try_null_coalescing_if(
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

    if !exprs_equal(&then_target, &else_target) {
        return None;
    }

    if !expr_references_var(&then_value, &checked_var) {
        return None;
    }

    Some(Statement::Assign(
        then_target,
        Expr::Binary(
            Box::new(then_value),
            BinOp::NullCoalesce,
            Box::new(else_value),
        ),
    ))
}

/// if (x == null) { target = y; } else { target = x; } -> target = x ?? y
fn try_null_coalescing_if_reversed(
    cond: &Expr,
    then_block: &[Statement],
    else_block: &[Statement],
) -> Option<Statement> {
    let checked_var = extract_null_check_eq(cond)?;

    if then_block.len() != 1 || else_block.len() != 1 {
        return None;
    }

    let (then_target, then_value) = extract_assignment(&then_block[0])?;
    let (else_target, else_value) = extract_assignment(&else_block[0])?;

    if !exprs_equal(&then_target, &else_target) {
        return None;
    }

    if !expr_references_var(&else_value, &checked_var) {
        return None;
    }

    Some(Statement::Assign(
        then_target,
        Expr::Binary(
            Box::new(else_value),
            BinOp::NullCoalesce,
            Box::new(then_value),
        ),
    ))
}
