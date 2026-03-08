//! C# code emitter — converts AST nodes to formatted C# source strings.

pub mod expressions;
pub use expressions::emit_expr;

use crate::decompiler::ast::*;

const INDENT: &str = "    ";

pub fn emit_statements(stmts: &[Statement], indent: usize) -> String {
    stmts.iter().map(|s| emit_statement(s, indent)).collect()
}

pub fn emit_statement(stmt: &Statement, indent: usize) -> String {
    let prefix = INDENT.repeat(indent);
    match stmt {
        Statement::Return(None) => format!("{}return;\n", prefix),
        Statement::Return(Some(expr)) => format!("{}return {};\n", prefix, emit_expr(expr)),
        Statement::Expr(expr) => format!("{}{};\n", prefix, emit_expr(expr)),
        Statement::Assign(target, value) => {
            format!("{}{} = {};\n", prefix, emit_expr(target), emit_expr(value))
        }
        Statement::LocalDecl(ty, name, init) => {
            if let Some(val) = init {
                format!("{}{} {} = {};\n", prefix, ty, name, emit_expr(val))
            } else {
                format!("{}{} {};\n", prefix, ty, name)
            }
        }
        Statement::If(cond, then_block, else_block) => {
            let mut s = format!("{}if ({})\n{}{{\n", prefix, emit_expr(cond), prefix);
            s += &emit_statements(then_block, indent + 1);
            s += &format!("{}}}\n", prefix);
            if let Some(els) = else_block {
                s += &format!("{}else\n{}{{\n", prefix, prefix);
                s += &emit_statements(els, indent + 1);
                s += &format!("{}}}\n", prefix);
            }
            s
        }
        Statement::While(cond, body) => {
            let mut s = format!("{}while ({})\n{}{{\n", prefix, emit_expr(cond), prefix);
            s += &emit_statements(body, indent + 1);
            s += &format!("{}}}\n", prefix);
            s
        }
        Statement::DoWhile(body, cond) => {
            let mut s = format!("{}do\n{}{{\n", prefix, prefix);
            s += &emit_statements(body, indent + 1);
            s += &format!("{}}} while ({});\n", prefix, emit_expr(cond));
            s
        }
        Statement::For(init, cond, update, body) => {
            let init_str = emit_statement(init, 0)
                .trim()
                .trim_end_matches(';')
                .to_string();
            let update_str = emit_statement(update, 0)
                .trim()
                .trim_end_matches(';')
                .to_string();
            let mut s = format!(
                "{}for ({}; {}; {})\n{}{{\n",
                prefix,
                init_str,
                emit_expr(cond),
                update_str,
                prefix
            );
            s += &emit_statements(body, indent + 1);
            s += &format!("{}}}\n", prefix);
            s
        }
        Statement::ForEach(ty, name, expr, body) => {
            let mut s = format!(
                "{}foreach ({} {} in {})\n{}{{\n",
                prefix,
                ty,
                name,
                emit_expr(expr),
                prefix
            );
            s += &emit_statements(body, indent + 1);
            s += &format!("{}}}\n", prefix);
            s
        }
        Statement::Switch(expr, cases, default) => {
            let mut s = format!("{}switch ({})\n{}{{\n", prefix, emit_expr(expr), prefix);
            for (labels, body) in cases {
                for label in labels {
                    s += &format!("{}case {}:\n", INDENT.repeat(indent + 1), emit_expr(label));
                }
                s += &emit_statements(body, indent + 2);
            }
            if let Some(def) = default {
                s += &format!("{}default:\n", INDENT.repeat(indent + 1));
                s += &emit_statements(def, indent + 2);
            }
            s += &format!("{}}}\n", prefix);
            s
        }
        Statement::Try(try_block, catches, finally_block) => {
            let mut s = format!("{}try\n{}{{\n", prefix, prefix);
            s += &emit_statements(try_block, indent + 1);
            s += &format!("{}}}\n", prefix);
            for catch in catches {
                if let Some(var) = &catch.var_name {
                    s += &format!(
                        "{}catch ({} {})\n{}{{\n",
                        prefix, catch.exception_type, var, prefix
                    );
                } else {
                    s += &format!("{}catch ({})\n{}{{\n", prefix, catch.exception_type, prefix);
                }
                s += &emit_statements(&catch.body, indent + 1);
                s += &format!("{}}}\n", prefix);
            }
            if let Some(fin) = finally_block {
                s += &format!("{}finally\n{}{{\n", prefix, prefix);
                s += &emit_statements(fin, indent + 1);
                s += &format!("{}}}\n", prefix);
            }
            s
        }
        Statement::Throw(Some(expr)) => format!("{}throw {};\n", prefix, emit_expr(expr)),
        Statement::Throw(None) => format!("{}throw;\n", prefix),
        Statement::Break => format!("{}break;\n", prefix),
        Statement::Continue => format!("{}continue;\n", prefix),
        Statement::Using(decl, body) => {
            let decl_str = emit_statement(decl, 0)
                .trim()
                .trim_end_matches(';')
                .to_string();
            let mut s = format!("{}using ({})\n{}{{\n", prefix, decl_str, prefix);
            s += &emit_statements(body, indent + 1);
            s += &format!("{}}}\n", prefix);
            s
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_return_field() {
        let stmt = Statement::Return(Some(Expr::Field(Box::new(Expr::This), "_field".into())));
        assert_eq!(emit_statement(&stmt, 2), "        return _field;\n");
    }

    #[test]
    fn test_emit_method_call() {
        let stmt = Statement::Expr(Expr::Call(
            Some(Box::new(Expr::This)),
            "SetVerticesDirty".into(),
            vec![],
        ));
        assert_eq!(emit_statement(&stmt, 2), "        SetVerticesDirty();\n");
    }

    #[test]
    fn test_emit_binary() {
        let expr = Expr::Binary(
            Box::new(Expr::Arg(0, "x".into())),
            BinOp::Add,
            Box::new(Expr::Int(1)),
        );
        assert_eq!(emit_expr(&expr), "x + 1");
    }

    #[test]
    fn test_emit_if_else() {
        let stmt = Statement::If(
            Expr::Bool(true),
            vec![Statement::Return(Some(Expr::Int(1)))],
            Some(vec![Statement::Return(Some(Expr::Int(0)))]),
        );
        let code = emit_statement(&stmt, 1);
        assert!(code.contains("if (true)"));
        assert!(code.contains("return 1;"));
        assert!(code.contains("else"));
        assert!(code.contains("return 0;"));
    }
}
