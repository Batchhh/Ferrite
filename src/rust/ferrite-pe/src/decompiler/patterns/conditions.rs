use crate::decompiler::ast::*;

pub(super) fn simplify_conditions(stmts: Vec<Statement>) -> Vec<Statement> {
    stmts
        .into_iter()
        .map(simplify_statement_conditions)
        .collect()
}

fn simplify_statement_conditions(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(cond, then_block, else_block) => {
            let cond = simplify_expr(cond);
            let then_block = simplify_conditions(then_block);
            let else_block = else_block.map(simplify_conditions);
            Statement::If(cond, then_block, else_block)
        }
        Statement::While(cond, body) => {
            Statement::While(simplify_expr(cond), simplify_conditions(body))
        }
        Statement::DoWhile(body, cond) => {
            Statement::DoWhile(simplify_conditions(body), simplify_expr(cond))
        }
        Statement::For(init, cond, update, body) => {
            let init = Box::new(simplify_statement_conditions(*init));
            let update = Box::new(simplify_statement_conditions(*update));
            Statement::For(init, simplify_expr(cond), update, simplify_conditions(body))
        }
        Statement::ForEach(ty, name, collection, body) => Statement::ForEach(
            ty,
            name,
            simplify_expr(collection),
            simplify_conditions(body),
        ),
        Statement::Switch(expr, cases, default) => {
            let cases = cases
                .into_iter()
                .map(|(labels, body)| (labels, simplify_conditions(body)))
                .collect();
            let default = default.map(simplify_conditions);
            Statement::Switch(simplify_expr(expr), cases, default)
        }
        Statement::Try(try_body, catches, finally_block) => {
            let try_body = simplify_conditions(try_body);
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    exception_type: c.exception_type,
                    var_name: c.var_name,
                    body: simplify_conditions(c.body),
                })
                .collect();
            let finally_block = finally_block.map(simplify_conditions);
            Statement::Try(try_body, catches, finally_block)
        }
        Statement::Using(decl, body) => Statement::Using(
            Box::new(simplify_statement_conditions(*decl)),
            simplify_conditions(body),
        ),
        Statement::Assign(target, value) => {
            Statement::Assign(simplify_expr(target), simplify_expr(value))
        }
        Statement::Return(Some(expr)) => Statement::Return(Some(simplify_expr(expr))),
        Statement::Expr(expr) => Statement::Expr(simplify_expr(expr)),
        Statement::LocalDecl(ty, name, Some(expr)) => {
            Statement::LocalDecl(ty, name, Some(simplify_expr(expr)))
        }
        other => other,
    }
}

pub(super) fn simplify_expr(expr: Expr) -> Expr {
    match expr {
        // !(!(expr)) → expr
        Expr::Unary(UnaryOp::LogicalNot, inner) => {
            let inner = simplify_expr(*inner);
            match inner {
                Expr::Unary(UnaryOp::LogicalNot, double_inner) => *double_inner,
                other => Expr::Unary(UnaryOp::LogicalNot, Box::new(other)),
            }
        }
        // (expr == 0) where expr looks boolean → !expr
        Expr::Binary(left, BinOp::Eq, right) => {
            let left = simplify_expr(*left);
            let right = simplify_expr(*right);
            if is_zero(&right) && is_boolean_expr(&left) {
                Expr::Unary(UnaryOp::LogicalNot, Box::new(left))
            } else if is_zero(&left) && is_boolean_expr(&right) {
                Expr::Unary(UnaryOp::LogicalNot, Box::new(right))
            } else {
                Expr::Binary(Box::new(left), BinOp::Eq, Box::new(right))
            }
        }
        // (expr != 0) where expr looks boolean → expr
        Expr::Binary(left, BinOp::Ne, right) => {
            let left = simplify_expr(*left);
            let right = simplify_expr(*right);
            if is_zero(&right) && is_boolean_expr(&left) {
                left
            } else if is_zero(&left) && is_boolean_expr(&right) {
                right
            } else {
                Expr::Binary(Box::new(left), BinOp::Ne, Box::new(right))
            }
        }
        Expr::Binary(left, op, right) => Expr::Binary(
            Box::new(simplify_expr(*left)),
            op,
            Box::new(simplify_expr(*right)),
        ),
        Expr::Unary(op, inner) => Expr::Unary(op, Box::new(simplify_expr(*inner))),
        Expr::Ternary(cond, then_val, else_val) => Expr::Ternary(
            Box::new(simplify_expr(*cond)),
            Box::new(simplify_expr(*then_val)),
            Box::new(simplify_expr(*else_val)),
        ),
        Expr::Call(obj, name, args) => {
            let obj = obj.map(|o| Box::new(simplify_expr(*o)));
            let args = args.into_iter().map(simplify_expr).collect();
            Expr::Call(obj, name, args)
        }
        Expr::StaticCall(ty, name, args) => {
            let args = args.into_iter().map(simplify_expr).collect();
            Expr::StaticCall(ty, name, args)
        }
        Expr::NewObj(ty, args) => {
            let args = args.into_iter().map(simplify_expr).collect();
            Expr::NewObj(ty, args)
        }
        other => other,
    }
}

pub(super) fn is_zero(expr: &Expr) -> bool {
    matches!(expr, Expr::Int(0))
}

fn is_boolean_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Binary(_, BinOp::Eq, _)
            | Expr::Binary(_, BinOp::Ne, _)
            | Expr::Binary(_, BinOp::Lt, _)
            | Expr::Binary(_, BinOp::Gt, _)
            | Expr::Binary(_, BinOp::Le, _)
            | Expr::Binary(_, BinOp::Ge, _)
            | Expr::Binary(_, BinOp::LogicalAnd, _)
            | Expr::Binary(_, BinOp::LogicalOr, _)
            | Expr::Unary(UnaryOp::LogicalNot, _)
            | Expr::IsInst(_, _)
            | Expr::Bool(_)
    )
}
