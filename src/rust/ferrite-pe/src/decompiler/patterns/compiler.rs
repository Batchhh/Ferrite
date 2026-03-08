use crate::decompiler::ast::*;

// ---------------------------------------------------------------------------
// Pattern 5: Constructor Call Rewriting
// ---------------------------------------------------------------------------

/// Rewrite `.ctor()` calls to `base()` calls in constructor bodies.
pub(super) fn rewrite_ctor_calls(stmts: Vec<Statement>) -> Vec<Statement> {
    stmts
        .into_iter()
        .filter_map(|stmt| match stmt {
            Statement::Expr(Expr::Call(Some(obj), ref name, ref args)) if name == ".ctor" => {
                if matches!(obj.as_ref(), Expr::This) && args.is_empty() {
                    None
                } else {
                    Some(Statement::Expr(Expr::Call(
                        Some(Box::new(Expr::Raw("base".into()))),
                        name.clone(),
                        args.clone(),
                    )))
                }
            }
            Statement::Expr(Expr::StaticCall(_, ref name, ref args)) if name == ".ctor" => {
                if args.is_empty() {
                    None
                } else {
                    Some(Statement::Expr(Expr::Call(
                        Some(Box::new(Expr::Raw("base".into()))),
                        name.clone(),
                        args.clone(),
                    )))
                }
            }
            Statement::If(cond, then_b, else_b) => Some(Statement::If(
                cond,
                rewrite_ctor_calls(then_b),
                else_b.map(rewrite_ctor_calls),
            )),
            Statement::Try(try_b, catches, finally_b) => {
                let catches = catches
                    .into_iter()
                    .map(|c| CatchClause {
                        body: rewrite_ctor_calls(c.body),
                        ..c
                    })
                    .collect();
                Some(Statement::Try(
                    rewrite_ctor_calls(try_b),
                    catches,
                    finally_b.map(rewrite_ctor_calls),
                ))
            }
            other => Some(other),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Pattern 6: Compiler-Generated Code Simplification
// ---------------------------------------------------------------------------

pub(super) fn simplify_compiler_generated(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result = Vec::with_capacity(stmts.len());
    for stmt in stmts {
        let stmt = recurse_compiler_generated(stmt);
        if let Statement::If(ref cond, ref body, None) = stmt {
            if is_compiler_generated_null_check(cond) && !body.is_empty() {
                let mut kept = Vec::new();
                for s in body {
                    match s {
                        Statement::Assign(ref target, _) if is_compiler_generated_field(target) => {
                            kept.push(s.clone());
                        }
                        Statement::Expr(Expr::NewObj(..)) => {}
                        other => kept.push(other.clone()),
                    }
                }
                if !kept.is_empty() {
                    result.extend(kept);
                    continue;
                }
            }
        }
        result.push(stmt);
    }
    result
}

fn is_compiler_generated_null_check(expr: &Expr) -> bool {
    match expr {
        Expr::Unary(UnaryOp::LogicalNot, inner) => is_compiler_generated_field(inner),
        _ => false,
    }
}

pub(super) fn is_compiler_generated_field(expr: &Expr) -> bool {
    match expr {
        Expr::StaticField(ty, name) => ty.contains("<>") || name.contains("<>"),
        Expr::Field(_, name) => name.contains("<>"),
        _ => false,
    }
}

fn recurse_compiler_generated(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(cond, then_b, else_b) => Statement::If(
            cond,
            simplify_compiler_generated(then_b),
            else_b.map(simplify_compiler_generated),
        ),
        Statement::While(cond, body) => Statement::While(cond, simplify_compiler_generated(body)),
        Statement::DoWhile(body, cond) => {
            Statement::DoWhile(simplify_compiler_generated(body), cond)
        }
        Statement::For(init, cond, update, body) => {
            Statement::For(init, cond, update, simplify_compiler_generated(body))
        }
        Statement::ForEach(ty, name, coll, body) => {
            Statement::ForEach(ty, name, coll, simplify_compiler_generated(body))
        }
        Statement::Try(try_body, catches, finally_block) => {
            let try_body = simplify_compiler_generated(try_body);
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: simplify_compiler_generated(c.body),
                    ..c
                })
                .collect();
            let finally_block = finally_block.map(simplify_compiler_generated);
            Statement::Try(try_body, catches, finally_block)
        }
        Statement::Using(decl, body) => Statement::Using(decl, simplify_compiler_generated(body)),
        Statement::Switch(expr, cases, default) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, simplify_compiler_generated(b)))
                .collect();
            Statement::Switch(expr, cases, default.map(simplify_compiler_generated))
        }
        other => other,
    }
}

// ---------------------------------------------------------------------------
// Pattern 7: Self-field prefix removal
// ---------------------------------------------------------------------------

pub(super) fn simplify_self_references(
    stmts: Vec<Statement>,
    enclosing_type: &str,
) -> Vec<Statement> {
    if enclosing_type.is_empty() {
        return stmts;
    }
    stmts
        .into_iter()
        .map(|s| simplify_self_stmt(s, enclosing_type))
        .collect()
}

fn simplify_self_stmt(stmt: Statement, et: &str) -> Statement {
    match stmt {
        Statement::Expr(expr) => Statement::Expr(simplify_self_expr(expr, et)),
        Statement::Return(Some(expr)) => Statement::Return(Some(simplify_self_expr(expr, et))),
        Statement::Return(None) => Statement::Return(None),
        Statement::Assign(target, value) => Statement::Assign(
            simplify_self_expr(target, et),
            simplify_self_expr(value, et),
        ),
        Statement::If(cond, then_b, else_b) => Statement::If(
            simplify_self_expr(cond, et),
            simplify_self_references(then_b, et),
            else_b.map(|b| simplify_self_references(b, et)),
        ),
        Statement::While(cond, body) => Statement::While(
            simplify_self_expr(cond, et),
            simplify_self_references(body, et),
        ),
        Statement::DoWhile(body, cond) => Statement::DoWhile(
            simplify_self_references(body, et),
            simplify_self_expr(cond, et),
        ),
        Statement::For(init, cond, update, body) => Statement::For(
            Box::new(simplify_self_stmt(*init, et)),
            simplify_self_expr(cond, et),
            Box::new(simplify_self_stmt(*update, et)),
            simplify_self_references(body, et),
        ),
        Statement::ForEach(ty, name, coll, body) => Statement::ForEach(
            ty,
            name,
            simplify_self_expr(coll, et),
            simplify_self_references(body, et),
        ),
        Statement::Try(try_body, catches, finally_block) => {
            let try_body = simplify_self_references(try_body, et);
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: simplify_self_references(c.body, et),
                    ..c
                })
                .collect();
            let finally_block = finally_block.map(|b| simplify_self_references(b, et));
            Statement::Try(try_body, catches, finally_block)
        }
        Statement::Using(decl, body) => Statement::Using(
            Box::new(simplify_self_stmt(*decl, et)),
            simplify_self_references(body, et),
        ),
        Statement::Switch(expr, cases, default) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, simplify_self_references(b, et)))
                .collect();
            Statement::Switch(
                simplify_self_expr(expr, et),
                cases,
                default.map(|b| simplify_self_references(b, et)),
            )
        }
        Statement::Throw(Some(expr)) => Statement::Throw(Some(simplify_self_expr(expr, et))),
        Statement::LocalDecl(ty, name, Some(expr)) => {
            Statement::LocalDecl(ty, name, Some(simplify_self_expr(expr, et)))
        }
        other => other,
    }
}

fn simplify_self_expr(expr: Expr, et: &str) -> Expr {
    match expr {
        Expr::StaticField(ref ty, ref field) if ty == et => {
            Expr::Field(Box::new(Expr::This), field.clone())
        }
        Expr::StaticCall(ref ty, ref method, ref args) if ty == et => {
            let args = args
                .iter()
                .cloned()
                .map(|a| simplify_self_expr(a, et))
                .collect();
            Expr::Call(Some(Box::new(Expr::This)), method.clone(), args)
        }
        Expr::Field(obj, name) => Expr::Field(Box::new(simplify_self_expr(*obj, et)), name),
        Expr::Call(Some(obj), name, args) => {
            let args = args
                .into_iter()
                .map(|a| simplify_self_expr(a, et))
                .collect();
            Expr::Call(Some(Box::new(simplify_self_expr(*obj, et))), name, args)
        }
        Expr::Call(None, name, args) => {
            let args = args
                .into_iter()
                .map(|a| simplify_self_expr(a, et))
                .collect();
            Expr::Call(None, name, args)
        }
        Expr::StaticCall(ty, name, args) => {
            let args = args
                .into_iter()
                .map(|a| simplify_self_expr(a, et))
                .collect();
            Expr::StaticCall(ty, name, args)
        }
        Expr::Binary(lhs, op, rhs) => Expr::Binary(
            Box::new(simplify_self_expr(*lhs, et)),
            op,
            Box::new(simplify_self_expr(*rhs, et)),
        ),
        Expr::Unary(op, val) => Expr::Unary(op, Box::new(simplify_self_expr(*val, et))),
        Expr::NewObj(ty, args) => {
            let args = args
                .into_iter()
                .map(|a| simplify_self_expr(a, et))
                .collect();
            Expr::NewObj(ty, args)
        }
        Expr::Cast(ty, val) => Expr::Cast(ty, Box::new(simplify_self_expr(*val, et))),
        Expr::AsInst(val, ty) => Expr::AsInst(Box::new(simplify_self_expr(*val, et)), ty),
        Expr::IsInst(val, ty) => Expr::IsInst(Box::new(simplify_self_expr(*val, et)), ty),
        Expr::Ternary(c, t, e) => Expr::Ternary(
            Box::new(simplify_self_expr(*c, et)),
            Box::new(simplify_self_expr(*t, et)),
            Box::new(simplify_self_expr(*e, et)),
        ),
        Expr::ArrayElement(arr, idx) => Expr::ArrayElement(
            Box::new(simplify_self_expr(*arr, et)),
            Box::new(simplify_self_expr(*idx, et)),
        ),
        Expr::ArrayNew(ty, size) => Expr::ArrayNew(ty, Box::new(simplify_self_expr(*size, et))),
        Expr::ArrayLength(arr) => Expr::ArrayLength(Box::new(simplify_self_expr(*arr, et))),
        Expr::Box(ty, val) => Expr::Box(ty, Box::new(simplify_self_expr(*val, et))),
        Expr::Unbox(ty, val) => Expr::Unbox(ty, Box::new(simplify_self_expr(*val, et))),
        Expr::AddressOf(val) => Expr::AddressOf(Box::new(simplify_self_expr(*val, et))),
        other => other,
    }
}
