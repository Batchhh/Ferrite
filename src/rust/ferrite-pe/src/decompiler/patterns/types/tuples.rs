//! Tuple deconstruction pattern detection.
//!
//! Detects ValueTuple construction followed by Item1/Item2/... field reads
//! and rewrites to `var (x, y) = (a, b);`.

use crate::decompiler::ast::*;

/// Detect tuple deconstruction patterns and rewrite them.
pub fn detect_tuples(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result = Vec::with_capacity(stmts.len());
    let mut i = 0;
    let stmts_vec: Vec<Statement> = stmts.into_iter().map(recurse_tuples).collect();

    while i < stmts_vec.len() {
        if let Some((decon, consumed)) = try_detect_tuple(&stmts_vec, i) {
            result.push(decon);
            i += consumed;
        } else {
            result.push(stmts_vec[i].clone());
            i += 1;
        }
    }
    result
}

/// Try to detect a tuple deconstruction starting at `idx`.
fn try_detect_tuple(stmts: &[Statement], idx: usize) -> Option<(Statement, usize)> {
    let (temp_name, args) = extract_valuetuple_creation(&stmts[idx])?;
    let n = args.len();
    if n == 0 {
        return None;
    }

    // Check that the next N statements read Item1..ItemN from temp_name
    let mut vars = Vec::with_capacity(n);
    for item_idx in 0..n {
        let stmt = stmts.get(idx + 1 + item_idx)?;
        let field_name = format!("Item{}", item_idx + 1);
        let (ty, var_name) = extract_item_read(stmt, &temp_name, &field_name)?;
        vars.push((ty, var_name));
    }

    let tuple_expr = Expr::TupleExpr(args);
    let consumed = 1 + n;
    Some((Statement::TupleDeconstruct(vars, tuple_expr), consumed))
}

/// Extract temp var name and args from a ValueTuple creation.
fn extract_valuetuple_creation(stmt: &Statement) -> Option<(String, Vec<Expr>)> {
    match stmt {
        Statement::LocalDecl(_, name, Some(Expr::NewObj(ty, args)))
            if ty.starts_with("ValueTuple") =>
        {
            Some((name.clone(), args.clone()))
        }
        Statement::Assign(Expr::Local(_, name), Expr::NewObj(ty, args))
            if ty.starts_with("ValueTuple") =>
        {
            Some((name.clone(), args.clone()))
        }
        _ => None,
    }
}

/// Extract (type, var_name) from a statement that reads a field from a temp.
fn extract_item_read(
    stmt: &Statement,
    temp_name: &str,
    field_name: &str,
) -> Option<(String, String)> {
    match stmt {
        Statement::LocalDecl(ty, var_name, Some(Expr::Field(obj, fname)))
            if fname == field_name && is_local_named(obj, temp_name) =>
        {
            Some((ty.clone(), var_name.clone()))
        }
        Statement::Assign(Expr::Local(_, var_name), Expr::Field(obj, fname))
            if fname == field_name && is_local_named(obj, temp_name) =>
        {
            Some(("var".to_string(), var_name.clone()))
        }
        _ => None,
    }
}

/// Check if an expression is a Local with the given name.
fn is_local_named(expr: &Expr, name: &str) -> bool {
    matches!(expr, Expr::Local(_, n) if n == name)
}

/// Recurse into all statement variants.
fn recurse_tuples(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(c, tb, eb) => Statement::If(c, detect_tuples(tb), eb.map(detect_tuples)),
        Statement::While(c, b) => Statement::While(c, detect_tuples(b)),
        Statement::DoWhile(b, c) => Statement::DoWhile(detect_tuples(b), c),
        Statement::For(i, c, u, b) => Statement::For(i, c, u, detect_tuples(b)),
        Statement::ForEach(t, n, col, b) => Statement::ForEach(t, n, col, detect_tuples(b)),
        Statement::Try(tb, catches, fb) => {
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_tuples(c.body),
                    ..c
                })
                .collect();
            Statement::Try(detect_tuples(tb), catches, fb.map(detect_tuples))
        }
        Statement::Using(d, b) => Statement::Using(d, detect_tuples(b)),
        Statement::Lock(e, b) => Statement::Lock(e, detect_tuples(b)),
        Statement::Checked(b) => Statement::Checked(detect_tuples(b)),
        Statement::Unchecked(b) => Statement::Unchecked(detect_tuples(b)),
        Statement::Fixed(t, n, e, b) => Statement::Fixed(t, n, e, detect_tuples(b)),
        Statement::Switch(e, cases, def) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_tuples(b)))
                .collect();
            Statement::Switch(e, cases, def.map(detect_tuples))
        }
        other => other,
    }
}
