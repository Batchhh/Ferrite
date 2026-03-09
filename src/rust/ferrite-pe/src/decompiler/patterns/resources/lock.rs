//! Lock statement pattern detection.
//!
//! Rewrites `Monitor.Enter/Exit` + `try/finally` into `lock (obj) { body }`.

use crate::decompiler::ast::*;

pub fn detect_lock(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result = Vec::with_capacity(stmts.len());
    let mut i = 0;
    while i < stmts.len() {
        if let Some((lock_stmt, consumed)) = try_detect_lock_at(&stmts, i) {
            result.push(lock_stmt);
            i += consumed;
        } else {
            result.push(recurse_lock(stmts[i].clone()));
            i += 1;
        }
    }
    result
}

fn recurse_lock(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(c, tb, eb) => Statement::If(c, detect_lock(tb), eb.map(detect_lock)),
        Statement::While(c, b) => Statement::While(c, detect_lock(b)),
        Statement::DoWhile(b, c) => Statement::DoWhile(detect_lock(b), c),
        Statement::For(i, c, u, b) => Statement::For(i, c, u, detect_lock(b)),
        Statement::ForEach(t, n, col, b) => Statement::ForEach(t, n, col, detect_lock(b)),
        Statement::Try(tb, catches, fb) => {
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_lock(c.body),
                    ..c
                })
                .collect();
            Statement::Try(detect_lock(tb), catches, fb.map(detect_lock))
        }
        Statement::Using(d, b) => Statement::Using(d, detect_lock(b)),
        Statement::Lock(e, b) => Statement::Lock(e, detect_lock(b)),
        Statement::Checked(b) => Statement::Checked(detect_lock(b)),
        Statement::Unchecked(b) => Statement::Unchecked(detect_lock(b)),
        Statement::Fixed(t, n, e, b) => Statement::Fixed(t, n, e, detect_lock(b)),
        Statement::Switch(e, cases, def) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_lock(b)))
                .collect();
            Statement::Switch(e, cases, def.map(detect_lock))
        }
        other => other,
    }
}

/// Try to detect a lock pattern starting at `idx`.
fn try_detect_lock_at(stmts: &[Statement], idx: usize) -> Option<(Statement, usize)> {
    // Pattern 1: C# 4+ — LocalDecl(bool, false) + Try(Enter+body, finally(if+Exit))
    if let Some(result) = try_csharp4_lock(stmts, idx) {
        return Some(result);
    }
    // Pattern 2: Pre-C# 4 — Monitor.Enter(obj) + Try(body, finally(Exit))
    try_pre_csharp4_lock(stmts, idx)
}

/// C# 4+: `bool V = false; try { Monitor.Enter(obj, ref V); body } finally { if (V) Exit(obj); }`
fn try_csharp4_lock(stmts: &[Statement], idx: usize) -> Option<(Statement, usize)> {
    let decl = stmts.get(idx)?;
    match decl {
        Statement::LocalDecl(_, _, Some(Expr::Bool(false))) => {}
        _ => return None,
    }
    let try_stmt = stmts.get(idx + 1)?;
    let (try_body, catches, finally) = match try_stmt {
        Statement::Try(tb, c, f) => (tb, c, f),
        _ => return None,
    };
    if !catches.is_empty() {
        return None;
    }
    let finally = finally.as_ref()?;
    if !finally_has_monitor_exit(finally) {
        return None;
    }
    let (obj, body) = extract_enter_and_body(try_body)?;
    let body = detect_lock(body);
    Some((Statement::Lock(Box::new(obj), body), 2))
}

/// Pre-C# 4: `Monitor.Enter(obj); try { body } finally { Monitor.Exit(obj); }`
fn try_pre_csharp4_lock(stmts: &[Statement], idx: usize) -> Option<(Statement, usize)> {
    let enter = stmts.get(idx)?;
    let obj = extract_monitor_enter_single(enter)?;
    let try_stmt = stmts.get(idx + 1)?;
    let (try_body, catches, finally) = match try_stmt {
        Statement::Try(tb, c, f) => (tb, c, f),
        _ => return None,
    };
    if !catches.is_empty() {
        return None;
    }
    let finally = finally.as_ref()?;
    if !finally_has_monitor_exit(finally) {
        return None;
    }
    let body = detect_lock(try_body.clone());
    Some((Statement::Lock(Box::new(obj), body), 2))
}

/// Extract the lock object from `Monitor.Enter(obj)` (single-arg form).
fn extract_monitor_enter_single(stmt: &Statement) -> Option<Expr> {
    match stmt {
        Statement::Expr(Expr::StaticCall(ty, method, args))
            if is_monitor(ty) && method == "Enter" && args.len() == 1 =>
        {
            Some(args[0].clone())
        }
        Statement::Expr(Expr::Call(_, method, args)) if method == "Enter" && args.len() == 1 => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

/// From a try body starting with Monitor.Enter(obj, ref flag), return (obj, rest).
fn extract_enter_and_body(try_body: &[Statement]) -> Option<(Expr, Vec<Statement>)> {
    let first = try_body.first()?;
    let obj = match first {
        Statement::Expr(Expr::StaticCall(ty, method, args))
            if is_monitor(ty) && method == "Enter" && args.len() == 2 =>
        {
            args[0].clone()
        }
        Statement::Expr(Expr::Call(_, method, args)) if method == "Enter" && args.len() == 2 => {
            args[0].clone()
        }
        _ => return None,
    };
    Some((obj, try_body[1..].to_vec()))
}

fn finally_has_monitor_exit(finally: &[Statement]) -> bool {
    for stmt in finally {
        match stmt {
            Statement::Expr(Expr::StaticCall(ty, m, _)) if is_monitor(ty) && m == "Exit" => {
                return true;
            }
            Statement::Expr(Expr::Call(_, m, _)) if m == "Exit" => return true,
            Statement::If(_, then_b, _) => {
                if finally_has_monitor_exit(then_b) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

fn is_monitor(ty: &str) -> bool {
    ty == "Monitor" || ty == "System.Threading.Monitor"
}
