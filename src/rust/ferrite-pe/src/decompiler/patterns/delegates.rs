use super::compiler::is_compiler_generated_field;
use super::null_coalescing::exprs_equal;
use crate::decompiler::ast::*;

pub(super) fn propagate_delegate_assignments(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result: Vec<Statement> = Vec::with_capacity(stmts.len());
    let mut i = 0;
    let stmts_vec: Vec<Statement> = stmts;
    while i < stmts_vec.len() {
        if i + 1 < stmts_vec.len() {
            if let Statement::Assign(ref target, ref value) = stmts_vec[i] {
                if is_compiler_generated_field(target) {
                    let next = &stmts_vec[i + 1];
                    if expr_appears_in_stmt(target, next) {
                        let substituted = substitute_in_stmt(target, value, next.clone());
                        result.push(substituted);
                        i += 2;
                        continue;
                    }
                }
            }
        }
        let stmt = recurse_propagate(stmts_vec[i].clone());
        result.push(stmt);
        i += 1;
    }
    result
}

fn recurse_propagate(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(cond, then_b, else_b) => Statement::If(
            cond,
            propagate_delegate_assignments(then_b),
            else_b.map(propagate_delegate_assignments),
        ),
        Statement::While(cond, body) => {
            Statement::While(cond, propagate_delegate_assignments(body))
        }
        Statement::DoWhile(body, cond) => {
            Statement::DoWhile(propagate_delegate_assignments(body), cond)
        }
        Statement::For(init, cond, update, body) => {
            Statement::For(init, cond, update, propagate_delegate_assignments(body))
        }
        Statement::ForEach(ty, name, coll, body) => {
            Statement::ForEach(ty, name, coll, propagate_delegate_assignments(body))
        }
        Statement::Try(try_body, catches, finally_block) => {
            let try_body = propagate_delegate_assignments(try_body);
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: propagate_delegate_assignments(c.body),
                    ..c
                })
                .collect();
            let finally_block = finally_block.map(propagate_delegate_assignments);
            Statement::Try(try_body, catches, finally_block)
        }
        Statement::Using(decl, body) => {
            Statement::Using(decl, propagate_delegate_assignments(body))
        }
        Statement::Switch(expr, cases, default) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, propagate_delegate_assignments(b)))
                .collect();
            Statement::Switch(expr, cases, default.map(propagate_delegate_assignments))
        }
        other => other,
    }
}

fn expr_appears_in_stmt(target: &Expr, stmt: &Statement) -> bool {
    match stmt {
        Statement::Expr(e) => expr_appears_in(target, e),
        Statement::Return(Some(e)) => expr_appears_in(target, e),
        Statement::Assign(t, v) => expr_appears_in(target, t) || expr_appears_in(target, v),
        Statement::LocalDecl(_, _, Some(e)) => expr_appears_in(target, e),
        Statement::If(cond, then_b, else_b) => {
            expr_appears_in(target, cond)
                || then_b.iter().any(|s| expr_appears_in_stmt(target, s))
                || else_b
                    .as_ref()
                    .is_some_and(|b| b.iter().any(|s| expr_appears_in_stmt(target, s)))
        }
        _ => false,
    }
}

fn expr_appears_in(target: &Expr, expr: &Expr) -> bool {
    if exprs_match(target, expr) {
        return true;
    }
    match expr {
        Expr::Call(obj, _, args) => {
            obj.as_ref().is_some_and(|o| expr_appears_in(target, o))
                || args.iter().any(|a| expr_appears_in(target, a))
        }
        Expr::StaticCall(_, _, args) => args.iter().any(|a| expr_appears_in(target, a)),
        Expr::NewObj(_, args) => args.iter().any(|a| expr_appears_in(target, a)),
        Expr::Binary(l, _, r) => expr_appears_in(target, l) || expr_appears_in(target, r),
        Expr::Unary(_, inner) => expr_appears_in(target, inner),
        Expr::Ternary(c, t, e) => {
            expr_appears_in(target, c) || expr_appears_in(target, t) || expr_appears_in(target, e)
        }
        Expr::Field(obj, _) => expr_appears_in(target, obj),
        Expr::ArrayElement(arr, idx) => {
            expr_appears_in(target, arr) || expr_appears_in(target, idx)
        }
        Expr::Cast(_, inner) | Expr::IsInst(inner, _) | Expr::AsInst(inner, _) => {
            expr_appears_in(target, inner)
        }
        Expr::Box(_, inner) | Expr::Unbox(_, inner) => expr_appears_in(target, inner),
        Expr::AddressOf(inner) => expr_appears_in(target, inner),
        Expr::ArrayLength(arr) => expr_appears_in(target, arr),
        _ => false,
    }
}

fn exprs_match(a: &Expr, b: &Expr) -> bool {
    match (a, b) {
        (Expr::StaticField(t1, n1), Expr::StaticField(t2, n2)) => t1 == t2 && n1 == n2,
        (Expr::Field(o1, n1), Expr::Field(o2, n2)) => n1 == n2 && exprs_equal(o1, o2),
        _ => false,
    }
}

fn substitute_in_stmt(target: &Expr, replacement: &Expr, stmt: Statement) -> Statement {
    match stmt {
        Statement::Expr(e) => Statement::Expr(substitute_in_expr(target, replacement, e)),
        Statement::Return(Some(e)) => {
            Statement::Return(Some(substitute_in_expr(target, replacement, e)))
        }
        Statement::Return(None) => Statement::Return(None),
        Statement::Assign(t, v) => Statement::Assign(
            substitute_in_expr(target, replacement, t),
            substitute_in_expr(target, replacement, v),
        ),
        Statement::LocalDecl(ty, name, init) => Statement::LocalDecl(
            ty,
            name,
            init.map(|e| substitute_in_expr(target, replacement, e)),
        ),
        Statement::If(cond, then_b, else_b) => Statement::If(
            substitute_in_expr(target, replacement, cond),
            then_b
                .into_iter()
                .map(|s| substitute_in_stmt(target, replacement, s))
                .collect(),
            else_b.map(|b| {
                b.into_iter()
                    .map(|s| substitute_in_stmt(target, replacement, s))
                    .collect()
            }),
        ),
        other => other,
    }
}

fn substitute_in_expr(target: &Expr, replacement: &Expr, expr: Expr) -> Expr {
    if exprs_match(target, &expr) {
        return replacement.clone();
    }
    match expr {
        Expr::Call(obj, name, args) => {
            let obj = obj.map(|o| Box::new(substitute_in_expr(target, replacement, *o)));
            let args = args
                .into_iter()
                .map(|a| substitute_in_expr(target, replacement, a))
                .collect();
            Expr::Call(obj, name, args)
        }
        Expr::StaticCall(ty, name, args) => {
            let args = args
                .into_iter()
                .map(|a| substitute_in_expr(target, replacement, a))
                .collect();
            Expr::StaticCall(ty, name, args)
        }
        Expr::NewObj(ty, args) => {
            let args = args
                .into_iter()
                .map(|a| substitute_in_expr(target, replacement, a))
                .collect();
            Expr::NewObj(ty, args)
        }
        Expr::Binary(l, op, r) => Expr::Binary(
            Box::new(substitute_in_expr(target, replacement, *l)),
            op,
            Box::new(substitute_in_expr(target, replacement, *r)),
        ),
        Expr::Unary(op, inner) => Expr::Unary(
            op,
            Box::new(substitute_in_expr(target, replacement, *inner)),
        ),
        Expr::Ternary(c, t, e) => Expr::Ternary(
            Box::new(substitute_in_expr(target, replacement, *c)),
            Box::new(substitute_in_expr(target, replacement, *t)),
            Box::new(substitute_in_expr(target, replacement, *e)),
        ),
        Expr::Field(obj, name) => Expr::Field(
            Box::new(substitute_in_expr(target, replacement, *obj)),
            name,
        ),
        Expr::ArrayElement(arr, idx) => Expr::ArrayElement(
            Box::new(substitute_in_expr(target, replacement, *arr)),
            Box::new(substitute_in_expr(target, replacement, *idx)),
        ),
        Expr::Cast(ty, inner) => Expr::Cast(
            ty,
            Box::new(substitute_in_expr(target, replacement, *inner)),
        ),
        Expr::IsInst(inner, ty) => Expr::IsInst(
            Box::new(substitute_in_expr(target, replacement, *inner)),
            ty,
        ),
        Expr::AsInst(inner, ty) => Expr::AsInst(
            Box::new(substitute_in_expr(target, replacement, *inner)),
            ty,
        ),
        Expr::Box(ty, inner) => Expr::Box(
            ty,
            Box::new(substitute_in_expr(target, replacement, *inner)),
        ),
        Expr::Unbox(ty, inner) => Expr::Unbox(
            ty,
            Box::new(substitute_in_expr(target, replacement, *inner)),
        ),
        Expr::AddressOf(inner) => {
            Expr::AddressOf(Box::new(substitute_in_expr(target, replacement, *inner)))
        }
        Expr::ArrayLength(arr) => {
            Expr::ArrayLength(Box::new(substitute_in_expr(target, replacement, *arr)))
        }
        other => other,
    }
}
