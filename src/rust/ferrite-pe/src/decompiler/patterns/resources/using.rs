use super::super::helpers::{expr_var_name, finally_calls_dispose};
use crate::decompiler::ast::*;

pub fn detect_using(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result = Vec::with_capacity(stmts.len());
    let mut i = 0;
    while i < stmts.len() {
        if let Some((using_stmt, consumed)) = try_detect_using_at(&stmts, i) {
            result.push(using_stmt);
            i += consumed;
        } else {
            result.push(recurse_using(stmts[i].clone()));
            i += 1;
        }
    }
    result
}

fn recurse_using(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(cond, then_b, else_b) => {
            Statement::If(cond, detect_using(then_b), else_b.map(detect_using))
        }
        Statement::While(cond, body) => Statement::While(cond, detect_using(body)),
        Statement::DoWhile(body, cond) => Statement::DoWhile(detect_using(body), cond),
        Statement::For(init, cond, update, body) => {
            Statement::For(init, cond, update, detect_using(body))
        }
        Statement::ForEach(ty, name, coll, body) => {
            Statement::ForEach(ty, name, coll, detect_using(body))
        }
        Statement::Try(try_body, catches, finally_block) => {
            let try_body = detect_using(try_body);
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_using(c.body),
                    ..c
                })
                .collect();
            let finally_block = finally_block.map(detect_using);
            Statement::Try(try_body, catches, finally_block)
        }
        Statement::Using(decl, body) => Statement::Using(decl, detect_using(body)),
        Statement::Switch(expr, cases, default) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_using(b)))
                .collect();
            Statement::Switch(expr, cases, default.map(detect_using))
        }
        other => other,
    }
}

fn try_detect_using_at(stmts: &[Statement], idx: usize) -> Option<(Statement, usize)> {
    let mut resource_decl = None;
    let mut resource_var = None;
    let mut try_idx = idx;

    if let Some(stmt) = stmts.get(idx) {
        match stmt {
            Statement::Assign(target, _value) => {
                if let Some(var) = expr_var_name(target) {
                    resource_var = Some(var);
                    resource_decl = Some(stmt.clone());
                    try_idx = idx + 1;
                }
            }
            Statement::LocalDecl(_ty, name, Some(_value)) => {
                resource_var = Some(name.clone());
                resource_decl = Some(stmt.clone());
                try_idx = idx + 1;
            }
            _ => {}
        }
    }

    let try_stmt = stmts.get(try_idx)?;
    let (try_body, catches, finally_block) = match try_stmt {
        Statement::Try(try_body, catches, finally_block) => (try_body, catches, finally_block),
        _ => return None,
    };

    if !catches.is_empty() {
        return None;
    }
    let finally = finally_block.as_ref()?;

    if !finally_disposes_var(finally, resource_var.as_deref()) {
        return None;
    }

    let decl = resource_decl?;

    let body = detect_using(try_body.clone());
    let consumed = try_idx - idx + 1;

    Some((Statement::Using(Box::new(decl), body), consumed))
}

fn finally_disposes_var(finally: &[Statement], _var_name: Option<&str>) -> bool {
    finally_calls_dispose(finally)
}
