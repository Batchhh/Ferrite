use super::super::helpers::{expr_var_name, finally_calls_dispose};
use super::conditions::is_zero;
use crate::decompiler::ast::*;

pub fn detect_foreach(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result = Vec::with_capacity(stmts.len());
    let mut i = 0;
    while i < stmts.len() {
        if let Some((foreach_stmt, consumed)) = try_detect_foreach_at(&stmts, i) {
            result.push(foreach_stmt);
            i += consumed;
        } else {
            result.push(recurse_foreach(stmts[i].clone()));
            i += 1;
        }
    }
    result
}

fn recurse_foreach(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(cond, then_b, else_b) => {
            Statement::If(cond, detect_foreach(then_b), else_b.map(detect_foreach))
        }
        Statement::While(cond, body) => Statement::While(cond, detect_foreach(body)),
        Statement::DoWhile(body, cond) => Statement::DoWhile(detect_foreach(body), cond),
        Statement::For(init, cond, update, body) => {
            Statement::For(init, cond, update, detect_foreach(body))
        }
        Statement::Try(try_body, catches, finally_block) => {
            let try_body = detect_foreach(try_body);
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_foreach(c.body),
                    ..c
                })
                .collect();
            let finally_block = finally_block.map(detect_foreach);
            Statement::Try(try_body, catches, finally_block)
        }
        Statement::Using(decl, body) => Statement::Using(decl, detect_foreach(body)),
        Statement::Switch(expr, cases, default) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_foreach(b)))
                .collect();
            Statement::Switch(expr, cases, default.map(detect_foreach))
        }
        other => other,
    }
}

fn try_detect_foreach_at(stmts: &[Statement], idx: usize) -> Option<(Statement, usize)> {
    let mut enumerator_var = None;
    let mut collection_expr = None;
    let mut try_idx = idx;

    if let Some(stmt) = stmts.get(idx) {
        if let Some((var, coll)) = extract_get_enumerator_assign(stmt) {
            enumerator_var = Some(var);
            collection_expr = Some(coll);
            try_idx = idx + 1;
        }
    }

    let try_stmt = stmts.get(try_idx)?;
    let (try_body, catches, finally_block) = match try_stmt {
        Statement::Try(try_body, catches, finally_block) => (try_body, catches, finally_block),
        _ => return None,
    };

    let finally = finally_block.as_ref()?;
    if !finally_calls_dispose(finally) {
        return None;
    }

    if try_body.is_empty() {
        return None;
    }

    let while_stmt = find_while_with_move_next(try_body)?;
    let (while_body, move_next_target) = while_stmt;

    if enumerator_var.is_none() {
        enumerator_var = move_next_target.clone();
    }

    if while_body.is_empty() {
        return None;
    }

    let (item_name, item_type, body_start) = extract_current_access(while_body)?;

    let collection = if let Some(coll) = collection_expr {
        coll
    } else if let Some(ref _var) = enumerator_var {
        Expr::Raw("/* collection */".into())
    } else {
        return None;
    };

    let foreach_body: Vec<Statement> = while_body[body_start..].to_vec();
    let foreach_body = detect_foreach(foreach_body);

    let consumed = try_idx - idx + 1;
    let _ = catches;
    Some((
        Statement::ForEach(item_type, item_name, collection, foreach_body),
        consumed,
    ))
}

fn extract_get_enumerator_assign(stmt: &Statement) -> Option<(String, Expr)> {
    match stmt {
        Statement::Assign(target, value) => {
            if let Some(collection) = extract_get_enumerator_call(value) {
                let var_name = expr_var_name(target)?;
                return Some((var_name, collection));
            }
            None
        }
        Statement::LocalDecl(_ty, name, Some(value)) => {
            if let Some(collection) = extract_get_enumerator_call(value) {
                return Some((name.clone(), collection));
            }
            None
        }
        _ => None,
    }
}

fn extract_get_enumerator_call(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Call(Some(obj), name, args)
            if args.is_empty() && (name == "GetEnumerator" || name == "get_enumerator") =>
        {
            Some(*obj.clone())
        }
        _ => None,
    }
}

fn find_while_with_move_next(stmts: &[Statement]) -> Option<(&Vec<Statement>, Option<String>)> {
    for stmt in stmts {
        if let Statement::While(cond, body) = stmt {
            if let Some(target) = calls_move_next(cond) {
                return Some((body, target));
            }
        }
    }
    None
}

fn calls_move_next(expr: &Expr) -> Option<Option<String>> {
    match expr {
        Expr::Call(Some(obj), name, args)
            if args.is_empty() && (name == "MoveNext" || name == "get_MoveNext") =>
        {
            Some(expr_var_name(obj))
        }
        Expr::Binary(left, BinOp::Ne, right) if is_zero(right) => calls_move_next(left),
        Expr::Unary(UnaryOp::LogicalNot, inner) => None.or_else(|| {
            if let Expr::Unary(UnaryOp::LogicalNot, inner2) = inner.as_ref() {
                calls_move_next(inner2)
            } else {
                None
            }
        }),
        _ => None,
    }
}

fn extract_current_access(body: &[Statement]) -> Option<(String, String, usize)> {
    if body.is_empty() {
        return None;
    }

    match &body[0] {
        Statement::Assign(target, value) => {
            if is_current_access(value) {
                let name = expr_var_name(target)?;
                return Some((name, "var".to_string(), 1));
            }
        }
        Statement::LocalDecl(ty, name, Some(value)) => {
            if is_current_access(value) {
                return Some((name.clone(), ty.clone(), 1));
            }
        }
        _ => {}
    }

    match &body[0] {
        Statement::Assign(target, Expr::Cast(ty, inner)) => {
            if is_current_access(inner) {
                let name = expr_var_name(target)?;
                return Some((name, ty.clone(), 1));
            }
        }
        Statement::Assign(target, Expr::Unbox(ty, inner)) => {
            if is_current_access(inner) {
                let name = expr_var_name(target)?;
                return Some((name, ty.clone(), 1));
            }
        }
        _ => {}
    }

    None
}

fn is_current_access(expr: &Expr) -> bool {
    match expr {
        Expr::Call(Some(_), name, args)
            if args.is_empty() && (name == "get_Current" || name == "Current") =>
        {
            true
        }
        Expr::Field(_, name) if name == "Current" => true,
        Expr::Cast(_, inner) | Expr::Unbox(_, inner) => is_current_access(inner),
        _ => false,
    }
}
