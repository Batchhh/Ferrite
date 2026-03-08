use crate::decompiler::ast::*;

pub(super) fn detect_for_loops(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result = Vec::with_capacity(stmts.len());
    let mut i = 0;
    while i < stmts.len() {
        if i + 1 < stmts.len() {
            if let Some(for_stmt) = try_build_for_loop(&stmts[i], &stmts[i + 1]) {
                result.push(for_stmt);
                i += 2;
                continue;
            }
        }
        result.push(recurse_for_loops(stmts[i].clone()));
        i += 1;
    }
    result
}

fn recurse_for_loops(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(cond, then_b, else_b) => {
            Statement::If(cond, detect_for_loops(then_b), else_b.map(detect_for_loops))
        }
        Statement::While(cond, body) => Statement::While(cond, detect_for_loops(body)),
        Statement::DoWhile(body, cond) => Statement::DoWhile(detect_for_loops(body), cond),
        Statement::For(init, cond, update, body) => {
            Statement::For(init, cond, update, detect_for_loops(body))
        }
        Statement::ForEach(ty, name, coll, body) => {
            Statement::ForEach(ty, name, coll, detect_for_loops(body))
        }
        Statement::Try(try_body, catches, finally_block) => {
            let try_body = detect_for_loops(try_body);
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_for_loops(c.body),
                    ..c
                })
                .collect();
            let finally_block = finally_block.map(detect_for_loops);
            Statement::Try(try_body, catches, finally_block)
        }
        Statement::Using(decl, body) => Statement::Using(decl, detect_for_loops(body)),
        Statement::Switch(expr, cases, default) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_for_loops(b)))
                .collect();
            Statement::Switch(expr, cases, default.map(detect_for_loops))
        }
        other => other,
    }
}

fn try_build_for_loop(init_stmt: &Statement, while_stmt: &Statement) -> Option<Statement> {
    let var_id = extract_init_var_id(init_stmt)?;

    let (cond, body) = match while_stmt {
        Statement::While(cond, body) => (cond, body),
        _ => return None,
    };

    if !expr_contains_var(&var_id, cond) {
        return None;
    }

    let last = body.last()?;
    if !is_var_increment_or_decrement(last, &var_id) {
        return None;
    }

    let for_body = detect_for_loops(body[..body.len() - 1].to_vec());

    Some(Statement::For(
        Box::new(init_stmt.clone()),
        cond.clone(),
        Box::new(last.clone()),
        for_body,
    ))
}

enum LoopVarId {
    Index(u16),
    Name(String),
}

fn extract_init_var_id(stmt: &Statement) -> Option<LoopVarId> {
    match stmt {
        Statement::Assign(Expr::Local(idx, _), Expr::Int(_)) => Some(LoopVarId::Index(*idx)),
        Statement::LocalDecl(_ty, name, Some(Expr::Int(_))) => Some(LoopVarId::Name(name.clone())),
        _ => None,
    }
}

fn expr_contains_var(var_id: &LoopVarId, expr: &Expr) -> bool {
    match var_id {
        LoopVarId::Index(idx) => expr_contains_local_idx(expr, *idx),
        LoopVarId::Name(name) => expr_contains_local_name(expr, name),
    }
}

fn expr_contains_local_name(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Local(_, n) => n == name,
        Expr::Binary(lhs, _, rhs) => {
            expr_contains_local_name(lhs, name) || expr_contains_local_name(rhs, name)
        }
        Expr::Unary(_, inner) => expr_contains_local_name(inner, name),
        Expr::Call(target, _, args) => {
            target
                .as_ref()
                .is_some_and(|t| expr_contains_local_name(t, name))
                || args.iter().any(|a| expr_contains_local_name(a, name))
        }
        Expr::StaticCall(_, _, args) => args.iter().any(|a| expr_contains_local_name(a, name)),
        Expr::Field(obj, _) => expr_contains_local_name(obj, name),
        Expr::ArrayElement(arr, index) => {
            expr_contains_local_name(arr, name) || expr_contains_local_name(index, name)
        }
        Expr::Cast(_, inner) => expr_contains_local_name(inner, name),
        Expr::IsInst(inner, _) | Expr::AsInst(inner, _) => expr_contains_local_name(inner, name),
        Expr::Ternary(c, t, e) => {
            expr_contains_local_name(c, name)
                || expr_contains_local_name(t, name)
                || expr_contains_local_name(e, name)
        }
        Expr::NewObj(_, args) => args.iter().any(|a| expr_contains_local_name(a, name)),
        Expr::Box(_, inner) | Expr::Unbox(_, inner) | Expr::AddressOf(inner) => {
            expr_contains_local_name(inner, name)
        }
        Expr::ArrayLength(arr) => expr_contains_local_name(arr, name),
        Expr::ArrayNew(_, size) => expr_contains_local_name(size, name),
        _ => false,
    }
}

pub(super) fn expr_contains_local_idx(expr: &Expr, idx: u16) -> bool {
    match expr {
        Expr::Local(i, _) => *i == idx,
        Expr::Binary(lhs, _, rhs) => {
            expr_contains_local_idx(lhs, idx) || expr_contains_local_idx(rhs, idx)
        }
        Expr::Unary(_, inner) => expr_contains_local_idx(inner, idx),
        Expr::Call(target, _, args) => {
            target
                .as_ref()
                .is_some_and(|t| expr_contains_local_idx(t, idx))
                || args.iter().any(|a| expr_contains_local_idx(a, idx))
        }
        Expr::StaticCall(_, _, args) => args.iter().any(|a| expr_contains_local_idx(a, idx)),
        Expr::Field(obj, _) => expr_contains_local_idx(obj, idx),
        Expr::ArrayElement(arr, index) => {
            expr_contains_local_idx(arr, idx) || expr_contains_local_idx(index, idx)
        }
        Expr::Cast(_, inner) => expr_contains_local_idx(inner, idx),
        Expr::IsInst(inner, _) | Expr::AsInst(inner, _) => expr_contains_local_idx(inner, idx),
        Expr::Ternary(c, t, e) => {
            expr_contains_local_idx(c, idx)
                || expr_contains_local_idx(t, idx)
                || expr_contains_local_idx(e, idx)
        }
        Expr::NewObj(_, args) => args.iter().any(|a| expr_contains_local_idx(a, idx)),
        Expr::Box(_, inner) | Expr::Unbox(_, inner) | Expr::AddressOf(inner) => {
            expr_contains_local_idx(inner, idx)
        }
        Expr::ArrayLength(arr) => expr_contains_local_idx(arr, idx),
        Expr::ArrayNew(_, size) => expr_contains_local_idx(size, idx),
        _ => false,
    }
}

fn is_var_increment_or_decrement(stmt: &Statement, var_id: &LoopVarId) -> bool {
    match stmt {
        Statement::Assign(Expr::Local(target_idx, target_name), Expr::Binary(lhs, op, rhs)) => {
            match var_id {
                LoopVarId::Index(idx) => {
                    if target_idx != idx {
                        return false;
                    }
                }
                LoopVarId::Name(name) => {
                    if target_name != name {
                        return false;
                    }
                }
            }
            match op {
                BinOp::Add | BinOp::Sub => {}
                _ => return false,
            }
            let lhs_is_var = match var_id {
                LoopVarId::Index(idx) => {
                    matches!(lhs.as_ref(), Expr::Local(i, _) if *i == *idx)
                }
                LoopVarId::Name(name) => {
                    matches!(lhs.as_ref(), Expr::Local(_, n) if n == name)
                }
            };
            let rhs_is_one = matches!(rhs.as_ref(), Expr::Int(1));
            if lhs_is_var && rhs_is_one {
                return true;
            }
            if matches!(op, BinOp::Add) {
                let lhs_is_one = matches!(lhs.as_ref(), Expr::Int(1));
                let rhs_is_var = match var_id {
                    LoopVarId::Index(idx) => {
                        matches!(rhs.as_ref(), Expr::Local(i, _) if *i == *idx)
                    }
                    LoopVarId::Name(name) => {
                        matches!(rhs.as_ref(), Expr::Local(_, n) if n == name)
                    }
                };
                if lhs_is_one && rhs_is_var {
                    return true;
                }
            }
            false
        }
        _ => false,
    }
}
