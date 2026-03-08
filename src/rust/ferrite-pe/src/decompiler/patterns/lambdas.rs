use super::LambdaMap;
use crate::decompiler::ast::*;

pub(super) fn inline_lambdas(stmts: Vec<Statement>, lambda_map: &LambdaMap) -> Vec<Statement> {
    if lambda_map.is_empty() {
        return stmts;
    }
    stmts
        .into_iter()
        .map(|s| inline_lambda_stmt(s, lambda_map))
        .collect()
}

fn inline_lambda_stmt(stmt: Statement, lm: &LambdaMap) -> Statement {
    match stmt {
        Statement::Expr(expr) => Statement::Expr(inline_lambda_expr(expr, lm)),
        Statement::Return(Some(expr)) => Statement::Return(Some(inline_lambda_expr(expr, lm))),
        Statement::Return(None) => Statement::Return(None),
        Statement::Assign(target, value) => Statement::Assign(
            inline_lambda_expr(target, lm),
            inline_lambda_expr(value, lm),
        ),
        Statement::LocalDecl(ty, name, init) => {
            Statement::LocalDecl(ty, name, init.map(|e| inline_lambda_expr(e, lm)))
        }
        Statement::If(cond, then_b, else_b) => Statement::If(
            inline_lambda_expr(cond, lm),
            inline_lambdas(then_b, lm),
            else_b.map(|b| inline_lambdas(b, lm)),
        ),
        Statement::While(cond, body) => {
            Statement::While(inline_lambda_expr(cond, lm), inline_lambdas(body, lm))
        }
        Statement::DoWhile(body, cond) => {
            Statement::DoWhile(inline_lambdas(body, lm), inline_lambda_expr(cond, lm))
        }
        Statement::For(init, cond, update, body) => Statement::For(
            Box::new(inline_lambda_stmt(*init, lm)),
            inline_lambda_expr(cond, lm),
            Box::new(inline_lambda_stmt(*update, lm)),
            inline_lambdas(body, lm),
        ),
        Statement::ForEach(ty, name, coll, body) => Statement::ForEach(
            ty,
            name,
            inline_lambda_expr(coll, lm),
            inline_lambdas(body, lm),
        ),
        Statement::Switch(expr, cases, default) => {
            let cases = cases
                .into_iter()
                .map(|(labels, body)| (labels, inline_lambdas(body, lm)))
                .collect();
            Statement::Switch(
                inline_lambda_expr(expr, lm),
                cases,
                default.map(|b| inline_lambdas(b, lm)),
            )
        }
        Statement::Try(try_body, catches, finally_block) => {
            let try_body = inline_lambdas(try_body, lm);
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: inline_lambdas(c.body, lm),
                    ..c
                })
                .collect();
            let finally_block = finally_block.map(|b| inline_lambdas(b, lm));
            Statement::Try(try_body, catches, finally_block)
        }
        Statement::Using(decl, body) => Statement::Using(
            Box::new(inline_lambda_stmt(*decl, lm)),
            inline_lambdas(body, lm),
        ),
        other => other,
    }
}

pub(super) fn is_delegate_type(ty: &str) -> bool {
    let base = ty.split('<').next().unwrap_or(ty);
    matches!(
        base,
        "Predicate"
            | "Func"
            | "Action"
            | "Comparison"
            | "Converter"
            | "EventHandler"
            | "AsyncCallback"
            | "ThreadStart"
            | "ParameterizedThreadStart"
            | "TimerCallback"
            | "WaitCallback"
            | "SendOrPostCallback"
            | "UnityAction"
    ) || base.ends_with("Handler")
        || base.ends_with("Callback")
        || base.ends_with("Action")
}

pub(super) fn extract_method_name_from_ref(raw: &str) -> Option<&str> {
    let s = raw.strip_prefix('&')?;
    if let Some(pos) = s.rfind("::") {
        Some(&s[pos + 2..])
    } else {
        Some(s)
    }
}

pub(super) fn inline_lambda_expr(expr: Expr, lm: &LambdaMap) -> Expr {
    match expr {
        Expr::NewObj(ref ty, ref args) if args.len() == 2 && is_delegate_type(ty) => {
            if let Expr::Raw(ref raw) = args[1] {
                if let Some(method_name) = extract_method_name_from_ref(raw) {
                    if let Some((params, body_stmts)) = lm.get(method_name) {
                        let lambda = build_lambda(params, body_stmts);
                        return lambda;
                    }
                }
            }
            let args = args
                .clone()
                .into_iter()
                .map(|a| inline_lambda_expr(a, lm))
                .collect();
            Expr::NewObj(ty.clone(), args)
        }
        Expr::Call(obj, name, args) => {
            let obj = obj.map(|o| Box::new(inline_lambda_expr(*o, lm)));
            let args = args
                .into_iter()
                .map(|a| inline_lambda_expr(a, lm))
                .collect();
            Expr::Call(obj, name, args)
        }
        Expr::StaticCall(ty, name, args) => {
            let args = args
                .into_iter()
                .map(|a| inline_lambda_expr(a, lm))
                .collect();
            Expr::StaticCall(ty, name, args)
        }
        Expr::NewObj(ty, args) => {
            let args = args
                .into_iter()
                .map(|a| inline_lambda_expr(a, lm))
                .collect();
            Expr::NewObj(ty, args)
        }
        Expr::Binary(left, op, right) => Expr::Binary(
            Box::new(inline_lambda_expr(*left, lm)),
            op,
            Box::new(inline_lambda_expr(*right, lm)),
        ),
        Expr::Unary(op, inner) => Expr::Unary(op, Box::new(inline_lambda_expr(*inner, lm))),
        Expr::Ternary(cond, then_val, else_val) => Expr::Ternary(
            Box::new(inline_lambda_expr(*cond, lm)),
            Box::new(inline_lambda_expr(*then_val, lm)),
            Box::new(inline_lambda_expr(*else_val, lm)),
        ),
        Expr::Field(obj, name) => Expr::Field(Box::new(inline_lambda_expr(*obj, lm)), name),
        Expr::ArrayElement(arr, idx) => Expr::ArrayElement(
            Box::new(inline_lambda_expr(*arr, lm)),
            Box::new(inline_lambda_expr(*idx, lm)),
        ),
        Expr::Cast(ty, inner) => Expr::Cast(ty, Box::new(inline_lambda_expr(*inner, lm))),
        Expr::IsInst(inner, ty) => Expr::IsInst(Box::new(inline_lambda_expr(*inner, lm)), ty),
        Expr::AsInst(inner, ty) => Expr::AsInst(Box::new(inline_lambda_expr(*inner, lm)), ty),
        Expr::Box(ty, inner) => Expr::Box(ty, Box::new(inline_lambda_expr(*inner, lm))),
        Expr::Unbox(ty, inner) => Expr::Unbox(ty, Box::new(inline_lambda_expr(*inner, lm))),
        Expr::AddressOf(inner) => Expr::AddressOf(Box::new(inline_lambda_expr(*inner, lm))),
        Expr::ArrayLength(arr) => Expr::ArrayLength(Box::new(inline_lambda_expr(*arr, lm))),
        other => other,
    }
}

fn build_lambda(params: &[(String, String)], body_stmts: &[Statement]) -> Expr {
    if body_stmts.len() == 1 {
        if let Statement::Return(Some(ref expr)) = body_stmts[0] {
            return Expr::Lambda(params.to_vec(), Box::new(LambdaBody::Expr(expr.clone())));
        }
    }
    Expr::Lambda(
        params.to_vec(),
        Box::new(LambdaBody::Block(body_stmts.to_vec())),
    )
}
