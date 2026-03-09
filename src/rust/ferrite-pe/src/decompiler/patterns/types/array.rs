use crate::decompiler::ast::*;

/// Detect `dup`+`stelem` array initialization patterns and fold them into
/// `ArrayInit` expressions.
pub fn detect_array_initializers(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result = Vec::with_capacity(stmts.len());
    let mut i = 0;
    let stmts_slice = &stmts;

    while i < stmts_slice.len() {
        if let Some((arr_type, elements, consumed, _expected_size)) =
            try_collect_array_init(stmts_slice, i)
        {
            let next_idx = i + consumed;
            if next_idx < stmts_slice.len() {
                let next_stmt = stmts_slice[next_idx].clone();
                let replaced = replace_array_new_with_init(next_stmt, &arr_type, &elements);
                if let Some(new_stmt) = replaced {
                    result.push(detect_array_init_in_stmt(new_stmt));
                    i = next_idx + 1;
                    continue;
                }
            }
        }
        result.push(detect_array_init_in_stmt(stmts_slice[i].clone()));
        i += 1;
    }

    result
}

fn detect_array_init_in_stmt(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(cond, then_b, else_b) => Statement::If(
            cond,
            detect_array_initializers(then_b),
            else_b.map(detect_array_initializers),
        ),
        Statement::While(cond, body) => Statement::While(cond, detect_array_initializers(body)),
        Statement::DoWhile(body, cond) => Statement::DoWhile(detect_array_initializers(body), cond),
        Statement::For(init, cond, upd, body) => {
            Statement::For(init, cond, upd, detect_array_initializers(body))
        }
        Statement::ForEach(ty, name, coll, body) => {
            Statement::ForEach(ty, name, coll, detect_array_initializers(body))
        }
        Statement::Try(try_b, catches, fin) => Statement::Try(
            detect_array_initializers(try_b),
            catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_array_initializers(c.body),
                    ..c
                })
                .collect(),
            fin.map(detect_array_initializers),
        ),
        Statement::Using(decl, body) => Statement::Using(decl, detect_array_initializers(body)),
        other => other,
    }
}

fn try_collect_array_init(
    stmts: &[Statement],
    start: usize,
) -> Option<(String, Vec<Expr>, usize, i64)> {
    let (arr_type, expected_size) = match &stmts[start] {
        Statement::Assign(Expr::ArrayElement(arr, idx), _value) => {
            if let (Expr::ArrayNew(ty, size), Expr::Int(0)) = (arr.as_ref(), idx.as_ref()) {
                if let Expr::Int(n) = size.as_ref() {
                    if *n > 0 && *n <= 64 {
                        (ty.clone(), *n)
                    } else {
                        return None;
                    }
                } else {
                    return None;
                }
            } else {
                return None;
            }
        }
        _ => return None,
    };

    let mut elements = Vec::new();
    let mut count = 0;

    for stmt in stmts.iter().skip(start) {
        match stmt {
            Statement::Assign(Expr::ArrayElement(arr, idx), value) => {
                if let (Expr::ArrayNew(ty, size), Expr::Int(elem_idx)) =
                    (arr.as_ref(), idx.as_ref())
                {
                    if let Expr::Int(n) = size.as_ref() {
                        if ty == &arr_type && *n == expected_size && *elem_idx == count as i64 {
                            elements.push(value.clone());
                            count += 1;
                            if count as i64 == expected_size {
                                break;
                            }
                            continue;
                        }
                    }
                }
                break;
            }
            _ => break,
        }
    }

    if count as i64 == expected_size {
        Some((arr_type, elements, count, expected_size))
    } else {
        None
    }
}

fn replace_array_new_with_init(
    stmt: Statement,
    arr_type: &str,
    elements: &[Expr],
) -> Option<Statement> {
    let mut found = false;
    let new_stmt = replace_array_new_in_stmt(stmt, arr_type, elements, &mut found);
    if found {
        Some(new_stmt)
    } else {
        None
    }
}

fn replace_array_new_in_stmt(
    stmt: Statement,
    arr_type: &str,
    elements: &[Expr],
    found: &mut bool,
) -> Statement {
    match stmt {
        Statement::Expr(e) => {
            Statement::Expr(replace_array_new_in_expr(e, arr_type, elements, found))
        }
        Statement::Return(Some(e)) => Statement::Return(Some(replace_array_new_in_expr(
            e, arr_type, elements, found,
        ))),
        Statement::Assign(t, v) => Statement::Assign(
            replace_array_new_in_expr(t, arr_type, elements, found),
            replace_array_new_in_expr(v, arr_type, elements, found),
        ),
        Statement::Throw(Some(e)) => Statement::Throw(Some(replace_array_new_in_expr(
            e, arr_type, elements, found,
        ))),
        Statement::LocalDecl(ty, name, Some(e)) => Statement::LocalDecl(
            ty,
            name,
            Some(replace_array_new_in_expr(e, arr_type, elements, found)),
        ),
        other => other,
    }
}

fn replace_array_new_in_expr(
    expr: Expr,
    arr_type: &str,
    elements: &[Expr],
    found: &mut bool,
) -> Expr {
    if *found {
        return expr;
    }
    match expr {
        Expr::ArrayNew(ref ty, ref size) => {
            if ty == arr_type {
                if let Expr::Int(n) = size.as_ref() {
                    if *n == elements.len() as i64 {
                        *found = true;
                        return Expr::ArrayInit(arr_type.to_string(), elements.to_vec());
                    }
                }
            }
            expr
        }
        Expr::Call(obj, name, args) => {
            let obj =
                obj.map(|o| Box::new(replace_array_new_in_expr(*o, arr_type, elements, found)));
            let args = args
                .into_iter()
                .map(|a| replace_array_new_in_expr(a, arr_type, elements, found))
                .collect();
            Expr::Call(obj, name, args)
        }
        Expr::StaticCall(ty, name, args) => {
            let args = args
                .into_iter()
                .map(|a| replace_array_new_in_expr(a, arr_type, elements, found))
                .collect();
            Expr::StaticCall(ty, name, args)
        }
        Expr::NewObj(ty, args) => {
            let args = args
                .into_iter()
                .map(|a| replace_array_new_in_expr(a, arr_type, elements, found))
                .collect();
            Expr::NewObj(ty, args)
        }
        Expr::Binary(l, op, r) => {
            let l = replace_array_new_in_expr(*l, arr_type, elements, found);
            let r = replace_array_new_in_expr(*r, arr_type, elements, found);
            Expr::Binary(Box::new(l), op, Box::new(r))
        }
        Expr::Unary(op, e) => Expr::Unary(
            op,
            Box::new(replace_array_new_in_expr(*e, arr_type, elements, found)),
        ),
        Expr::Cast(ty, e) => Expr::Cast(
            ty,
            Box::new(replace_array_new_in_expr(*e, arr_type, elements, found)),
        ),
        Expr::Field(obj, name) => Expr::Field(
            Box::new(replace_array_new_in_expr(*obj, arr_type, elements, found)),
            name,
        ),
        Expr::Ternary(c, t, e) => Expr::Ternary(
            Box::new(replace_array_new_in_expr(*c, arr_type, elements, found)),
            Box::new(replace_array_new_in_expr(*t, arr_type, elements, found)),
            Box::new(replace_array_new_in_expr(*e, arr_type, elements, found)),
        ),
        other => other,
    }
}
