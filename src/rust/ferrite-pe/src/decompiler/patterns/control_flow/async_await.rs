//! Async/await pattern detection.
//!
//! Detects the compiler-generated awaiter pattern and rewrites to `await`:
//! ```text
//! var awaiter = expr.GetAwaiter();
//! ... state machine checks ...
//! var result = awaiter.GetResult();
//! → var result = await expr;
//! ```

use crate::decompiler::ast::*;

/// Detect async/await patterns and rewrite to `await` expressions.
pub fn detect_async_await(stmts: Vec<Statement>) -> Vec<Statement> {
    let stmts_vec: Vec<Statement> = stmts.into_iter().map(recurse_async).collect();
    rewrite_awaiter_sequence(stmts_vec)
}

/// Scan statements for GetAwaiter/GetResult sequences and rewrite them.
fn rewrite_awaiter_sequence(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result = Vec::with_capacity(stmts.len());
    let mut i = 0;
    while i < stmts.len() {
        if let Some((awaiter_name, task_expr)) = extract_get_awaiter(&stmts[i]) {
            // Found GetAwaiter — look ahead for GetResult on same awaiter
            let (rewritten, consumed) = find_get_result(&stmts, i + 1, &awaiter_name, &task_expr);
            result.extend(rewritten);
            i += 1 + consumed;
        } else {
            result.push(stmts[i].clone());
            i += 1;
        }
    }
    result
}

/// Extract `(awaiter_name, task_expr)` from
/// `LocalDecl(_, name, Call(Some(task), "GetAwaiter", []))`.
fn extract_get_awaiter(stmt: &Statement) -> Option<(String, Expr)> {
    match stmt {
        Statement::LocalDecl(_, name, Some(Expr::Call(Some(obj), method, args)))
            if method == "GetAwaiter" && args.is_empty() =>
        {
            Some((name.clone(), obj.as_ref().clone()))
        }
        Statement::LocalDecl(_, name, Some(Expr::StaticCall(_, method, args)))
            if method == "GetAwaiter" && args.len() == 1 =>
        {
            Some((name.clone(), args[0].clone()))
        }
        _ => None,
    }
}

/// Look ahead from `start` for `awaiter.GetResult()`, skipping state machine
/// statements. Returns (replacement statements, number of statements consumed).
fn find_get_result(
    stmts: &[Statement],
    start: usize,
    awaiter_name: &str,
    task_expr: &Expr,
) -> (Vec<Statement>, usize) {
    let mut skipped = Vec::new();
    for (j, stmt) in stmts.iter().enumerate().skip(start) {
        // Check for `var result = awaiter.GetResult();`
        if let Some((ty, result_name)) = extract_get_result_decl(stmt, awaiter_name) {
            let await_stmt = Statement::LocalDecl(
                ty,
                result_name,
                Some(Expr::Await(Box::new(task_expr.clone()))),
            );
            let mut out = Vec::new();
            out.extend(skipped);
            out.push(await_stmt);
            return (out, j - start + 1);
        }
        // Check for void `awaiter.GetResult();`
        if is_void_get_result(stmt, awaiter_name) {
            let await_stmt = Statement::Expr(Expr::Await(Box::new(task_expr.clone())));
            let mut out = Vec::new();
            out.extend(skipped);
            out.push(await_stmt);
            return (out, j - start + 1);
        }
        // Skip state machine If blocks (IsCompleted checks etc.)
        if is_state_machine_stmt(stmt, awaiter_name) {
            continue;
        }
        // Non-matching statement — keep it but continue looking
        skipped.push(stmt.clone());
    }
    // No GetResult found — emit the original GetAwaiter as-is
    (Vec::new(), 0)
}

/// Extract `(type, result_name)` from
/// `LocalDecl(ty, name, Call(Some(Local(_, awaiter)), "GetResult", []))`.
fn extract_get_result_decl(stmt: &Statement, awaiter_name: &str) -> Option<(String, String)> {
    match stmt {
        Statement::LocalDecl(ty, name, Some(Expr::Call(Some(obj), method, args)))
            if method == "GetResult" && args.is_empty() =>
        {
            if matches!(obj.as_ref(), Expr::Local(_, n) if n == awaiter_name) {
                Some((ty.clone(), name.clone()))
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Check for void `awaiter.GetResult();` as expression statement.
fn is_void_get_result(stmt: &Statement, awaiter_name: &str) -> bool {
    matches!(
        stmt,
        Statement::Expr(Expr::Call(Some(obj), method, args))
            if method == "GetResult"
            && args.is_empty()
            && matches!(obj.as_ref(), Expr::Local(_, n) if n == awaiter_name)
    )
}

/// Check if a statement is a state machine check (e.g. `if (!awaiter.IsCompleted)`).
fn is_state_machine_stmt(stmt: &Statement, awaiter_name: &str) -> bool {
    match stmt {
        Statement::If(cond, _, _) => expr_references_awaiter(cond, awaiter_name),
        Statement::Assign(_, val) => expr_references_awaiter(val, awaiter_name),
        _ => false,
    }
}

/// Check if an expression references the awaiter variable.
fn expr_references_awaiter(expr: &Expr, awaiter_name: &str) -> bool {
    match expr {
        Expr::Local(_, name) => name == awaiter_name,
        Expr::Field(obj, _) => expr_references_awaiter(obj, awaiter_name),
        Expr::Call(Some(obj), _, args) => {
            expr_references_awaiter(obj, awaiter_name)
                || args
                    .iter()
                    .any(|a| expr_references_awaiter(a, awaiter_name))
        }
        Expr::Unary(_, inner) => expr_references_awaiter(inner, awaiter_name),
        Expr::Binary(l, _, r) => {
            expr_references_awaiter(l, awaiter_name) || expr_references_awaiter(r, awaiter_name)
        }
        _ => false,
    }
}

/// Recurse into all statement variants.
fn recurse_async(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(c, tb, eb) => {
            Statement::If(c, detect_async_await(tb), eb.map(detect_async_await))
        }
        Statement::While(c, b) => Statement::While(c, detect_async_await(b)),
        Statement::DoWhile(b, c) => Statement::DoWhile(detect_async_await(b), c),
        Statement::For(i, c, u, b) => Statement::For(i, c, u, detect_async_await(b)),
        Statement::ForEach(t, n, col, b) => Statement::ForEach(t, n, col, detect_async_await(b)),
        Statement::Try(tb, catches, fb) => {
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_async_await(c.body),
                    ..c
                })
                .collect();
            Statement::Try(detect_async_await(tb), catches, fb.map(detect_async_await))
        }
        Statement::Using(d, b) => Statement::Using(d, detect_async_await(b)),
        Statement::Lock(e, b) => Statement::Lock(e, detect_async_await(b)),
        Statement::Checked(b) => Statement::Checked(detect_async_await(b)),
        Statement::Unchecked(b) => Statement::Unchecked(detect_async_await(b)),
        Statement::Fixed(t, n, e, b) => Statement::Fixed(t, n, e, detect_async_await(b)),
        Statement::Switch(e, cases, def) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_async_await(b)))
                .collect();
            Statement::Switch(e, cases, def.map(detect_async_await))
        }
        other => other,
    }
}
