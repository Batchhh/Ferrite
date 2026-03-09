use crate::decompiler::ast::*;

/// Rewrite .NET property accessor calls to C# property syntax and
/// operator method calls to binary operator expressions.
pub fn rewrite_property_accessors(stmts: Vec<Statement>) -> Vec<Statement> {
    stmts.into_iter().map(rewrite_prop_stmt).collect()
}

pub fn rewrite_prop_stmt(stmt: Statement) -> Statement {
    match stmt {
        // Statement-level set_ rewriting: obj.set_X(value) → obj.X = value
        Statement::Expr(Expr::Call(Some(obj), ref name, ref args))
            if name.starts_with("set_") && args.len() == 1 =>
        {
            let prop_name = &name[4..];
            let obj = rewrite_prop_expr(*obj);
            let value = rewrite_prop_expr(args[0].clone());
            Statement::Assign(Expr::Field(Box::new(obj), prop_name.to_string()), value)
        }
        // Statement-level static set_ rewriting: Type.set_X(value) → Type.X = value
        Statement::Expr(Expr::StaticCall(ref ty, ref name, ref args))
            if name.starts_with("set_") && args.len() == 1 =>
        {
            let prop_name = &name[4..];
            let value = rewrite_prop_expr(args[0].clone());
            Statement::Assign(Expr::StaticField(ty.clone(), prop_name.to_string()), value)
        }
        Statement::Expr(expr) => Statement::Expr(rewrite_prop_expr(expr)),
        Statement::Return(Some(expr)) => Statement::Return(Some(rewrite_prop_expr(expr))),
        Statement::Assign(target, value) => {
            Statement::Assign(rewrite_prop_expr(target), rewrite_prop_expr(value))
        }
        Statement::If(cond, then_block, else_block) => Statement::If(
            rewrite_prop_expr(cond),
            rewrite_property_accessors(then_block),
            else_block.map(rewrite_property_accessors),
        ),
        Statement::While(cond, body) => {
            Statement::While(rewrite_prop_expr(cond), rewrite_property_accessors(body))
        }
        Statement::DoWhile(body, cond) => {
            Statement::DoWhile(rewrite_property_accessors(body), rewrite_prop_expr(cond))
        }
        Statement::For(init, cond, update, body) => {
            let init = Box::new(rewrite_prop_stmt(*init));
            let update = Box::new(rewrite_prop_stmt(*update));
            Statement::For(
                init,
                rewrite_prop_expr(cond),
                update,
                rewrite_property_accessors(body),
            )
        }
        Statement::ForEach(ty, name, coll, body) => Statement::ForEach(
            ty,
            name,
            rewrite_prop_expr(coll),
            rewrite_property_accessors(body),
        ),
        Statement::Switch(expr, cases, default) => {
            let cases = cases
                .into_iter()
                .map(|(labels, body)| {
                    let labels = labels.into_iter().map(rewrite_prop_expr).collect();
                    (labels, rewrite_property_accessors(body))
                })
                .collect();
            let default = default.map(rewrite_property_accessors);
            Statement::Switch(rewrite_prop_expr(expr), cases, default)
        }
        Statement::Try(try_body, catches, finally_block) => {
            let try_body = rewrite_property_accessors(try_body);
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    exception_type: c.exception_type,
                    var_name: c.var_name,
                    body: rewrite_property_accessors(c.body),
                })
                .collect();
            let finally_block = finally_block.map(rewrite_property_accessors);
            Statement::Try(try_body, catches, finally_block)
        }
        Statement::Using(decl, body) => Statement::Using(
            Box::new(rewrite_prop_stmt(*decl)),
            rewrite_property_accessors(body),
        ),
        Statement::Throw(Some(expr)) => Statement::Throw(Some(rewrite_prop_expr(expr))),
        Statement::LocalDecl(ty, name, Some(expr)) => {
            Statement::LocalDecl(ty, name, Some(rewrite_prop_expr(expr)))
        }
        other => other,
    }
}

pub fn rewrite_prop_expr(expr: Expr) -> Expr {
    match expr {
        // Instance property getter: obj.get_X() → obj.X
        Expr::Call(Some(obj), ref name, ref args)
            if name.starts_with("get_") && args.is_empty() =>
        {
            let prop_name = &name[4..];
            let obj = rewrite_prop_expr(*obj);
            Expr::Field(Box::new(obj), prop_name.to_string())
        }
        // Static property getter: Type.get_X() → Type.X
        Expr::StaticCall(ref ty, ref name, ref args)
            if name.starts_with("get_") && args.is_empty() =>
        {
            let prop_name = &name[4..];
            Expr::StaticField(ty.clone(), prop_name.to_string())
        }
        // Static op_Equality(a, b) → a == b
        Expr::StaticCall(_, ref name, ref args) if name == "op_Equality" && args.len() == 2 => {
            let a = rewrite_prop_expr(args[0].clone());
            let b = rewrite_prop_expr(args[1].clone());
            Expr::Binary(Box::new(a), BinOp::Eq, Box::new(b))
        }
        // Static op_Inequality(a, b) → a != b
        Expr::StaticCall(_, ref name, ref args) if name == "op_Inequality" && args.len() == 2 => {
            let a = rewrite_prop_expr(args[0].clone());
            let b = rewrite_prop_expr(args[1].clone());
            Expr::Binary(Box::new(a), BinOp::Ne, Box::new(b))
        }
        // Instance op_Equality(a, b) via Call(None, ...) → a == b
        Expr::Call(None, ref name, ref args) if name == "op_Equality" && args.len() == 2 => {
            let a = rewrite_prop_expr(args[0].clone());
            let b = rewrite_prop_expr(args[1].clone());
            Expr::Binary(Box::new(a), BinOp::Eq, Box::new(b))
        }
        // Instance op_Inequality(a, b) via Call(None, ...) → a != b
        Expr::Call(None, ref name, ref args) if name == "op_Inequality" && args.len() == 2 => {
            let a = rewrite_prop_expr(args[0].clone());
            let b = rewrite_prop_expr(args[1].clone());
            Expr::Binary(Box::new(a), BinOp::Ne, Box::new(b))
        }
        // Recurse into other expressions
        Expr::Call(obj, name, args) => {
            let obj = obj.map(|o| Box::new(rewrite_prop_expr(*o)));
            let args = args.into_iter().map(rewrite_prop_expr).collect();
            Expr::Call(obj, name, args)
        }
        Expr::StaticCall(ty, name, args) => {
            let args = args.into_iter().map(rewrite_prop_expr).collect();
            Expr::StaticCall(ty, name, args)
        }
        Expr::NewObj(ty, args) => {
            let args = args.into_iter().map(rewrite_prop_expr).collect();
            Expr::NewObj(ty, args)
        }
        Expr::Binary(left, op, right) => Expr::Binary(
            Box::new(rewrite_prop_expr(*left)),
            op,
            Box::new(rewrite_prop_expr(*right)),
        ),
        Expr::Unary(op, inner) => Expr::Unary(op, Box::new(rewrite_prop_expr(*inner))),
        Expr::Ternary(cond, then_val, else_val) => Expr::Ternary(
            Box::new(rewrite_prop_expr(*cond)),
            Box::new(rewrite_prop_expr(*then_val)),
            Box::new(rewrite_prop_expr(*else_val)),
        ),
        Expr::Field(obj, name) => Expr::Field(Box::new(rewrite_prop_expr(*obj)), name),
        Expr::Cast(ty, inner) => Expr::Cast(ty, Box::new(rewrite_prop_expr(*inner))),
        Expr::IsInst(inner, ty) => Expr::IsInst(Box::new(rewrite_prop_expr(*inner)), ty),
        Expr::AsInst(inner, ty) => Expr::AsInst(Box::new(rewrite_prop_expr(*inner)), ty),
        Expr::Box(ty, inner) => Expr::Box(ty, Box::new(rewrite_prop_expr(*inner))),
        Expr::Unbox(ty, inner) => Expr::Unbox(ty, Box::new(rewrite_prop_expr(*inner))),
        Expr::ArrayElement(arr, idx) => Expr::ArrayElement(
            Box::new(rewrite_prop_expr(*arr)),
            Box::new(rewrite_prop_expr(*idx)),
        ),
        Expr::ArrayNew(ty, size) => Expr::ArrayNew(ty, Box::new(rewrite_prop_expr(*size))),
        Expr::ArrayLength(arr) => Expr::ArrayLength(Box::new(rewrite_prop_expr(*arr))),
        Expr::AddressOf(inner) => Expr::AddressOf(Box::new(rewrite_prop_expr(*inner))),
        other => other,
    }
}
