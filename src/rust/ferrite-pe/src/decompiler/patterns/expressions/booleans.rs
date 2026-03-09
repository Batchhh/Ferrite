use crate::decompiler::ast::*;

pub fn coerce_booleans(stmts: Vec<Statement>) -> Vec<Statement> {
    stmts.into_iter().map(coerce_bool_stmt).collect()
}

fn looks_boolean(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower.starts_with("is")
        || lower.starts_with("has")
        || lower.starts_with("allow")
        || lower.starts_with("m_allow")
        || lower.starts_with("enabled")
        || lower.starts_with("active")
        || lower.starts_with("visible")
        || lower.starts_with("can")
        || lower.starts_with("should")
        || lower.starts_with("was")
        || lower.starts_with("did")
        || lower == "interactable"
        || lower == "enabled"
}

fn int_to_bool(expr: Expr) -> Expr {
    match expr {
        Expr::Int(0) => Expr::Bool(false),
        Expr::Int(1) => Expr::Bool(true),
        other => other,
    }
}

fn target_field_name(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Field(_, name) => Some(name.as_str()),
        Expr::StaticField(_, name) => Some(name.as_str()),
        _ => None,
    }
}

pub fn coerce_bool_stmt(stmt: Statement) -> Statement {
    match stmt {
        Statement::Assign(ref target, ref value) => {
            if let Some(name) = target_field_name(target) {
                if looks_boolean(name) {
                    return Statement::Assign(
                        coerce_bool_expr(target.clone()),
                        int_to_bool(coerce_bool_expr(value.clone())),
                    );
                }
            }
            Statement::Assign(
                coerce_bool_expr(target.clone()),
                coerce_bool_expr(value.clone()),
            )
        }
        Statement::If(cond, then_b, else_b) => Statement::If(
            int_to_bool(coerce_bool_expr(cond)),
            coerce_booleans(then_b),
            else_b.map(coerce_booleans),
        ),
        Statement::While(cond, body) => {
            Statement::While(int_to_bool(coerce_bool_expr(cond)), coerce_booleans(body))
        }
        Statement::DoWhile(body, cond) => {
            Statement::DoWhile(coerce_booleans(body), int_to_bool(coerce_bool_expr(cond)))
        }
        Statement::For(init, cond, update, body) => Statement::For(
            Box::new(coerce_bool_stmt(*init)),
            int_to_bool(coerce_bool_expr(cond)),
            Box::new(coerce_bool_stmt(*update)),
            coerce_booleans(body),
        ),
        Statement::ForEach(ty, name, coll, body) => {
            Statement::ForEach(ty, name, coerce_bool_expr(coll), coerce_booleans(body))
        }
        Statement::Try(try_body, catches, finally_block) => {
            let try_body = coerce_booleans(try_body);
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: coerce_booleans(c.body),
                    ..c
                })
                .collect();
            let finally_block = finally_block.map(coerce_booleans);
            Statement::Try(try_body, catches, finally_block)
        }
        Statement::Using(decl, body) => {
            Statement::Using(Box::new(coerce_bool_stmt(*decl)), coerce_booleans(body))
        }
        Statement::Switch(expr, cases, default) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, coerce_booleans(b)))
                .collect();
            Statement::Switch(coerce_bool_expr(expr), cases, default.map(coerce_booleans))
        }
        Statement::Expr(expr) => Statement::Expr(coerce_bool_expr(expr)),
        Statement::Return(Some(expr)) => Statement::Return(Some(coerce_bool_expr(expr))),
        Statement::LocalDecl(ty, name, Some(expr)) => {
            Statement::LocalDecl(ty, name, Some(coerce_bool_expr(expr)))
        }
        Statement::Throw(Some(expr)) => Statement::Throw(Some(coerce_bool_expr(expr))),
        other => other,
    }
}

fn coerce_bool_expr(expr: Expr) -> Expr {
    match expr {
        Expr::Binary(lhs, op, rhs) => Expr::Binary(
            Box::new(coerce_bool_expr(*lhs)),
            op,
            Box::new(coerce_bool_expr(*rhs)),
        ),
        Expr::Unary(op, val) => Expr::Unary(op, Box::new(coerce_bool_expr(*val))),
        Expr::Call(obj, name, args) => {
            let obj = obj.map(|o| Box::new(coerce_bool_expr(*o)));
            let args = args.into_iter().map(coerce_bool_expr).collect();
            Expr::Call(obj, name, args)
        }
        Expr::StaticCall(ty, name, args) => {
            let args = args.into_iter().map(coerce_bool_expr).collect();
            Expr::StaticCall(ty, name, args)
        }
        Expr::Ternary(c, t, e) => Expr::Ternary(
            Box::new(coerce_bool_expr(*c)),
            Box::new(coerce_bool_expr(*t)),
            Box::new(coerce_bool_expr(*e)),
        ),
        Expr::Field(obj, name) => Expr::Field(Box::new(coerce_bool_expr(*obj)), name),
        Expr::Cast(ty, val) => Expr::Cast(ty, Box::new(coerce_bool_expr(*val))),
        other => other,
    }
}
