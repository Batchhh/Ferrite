//! C# code emitter — converts AST nodes to formatted C# source strings.

pub mod expressions;
mod format;
pub mod statements;

pub use expressions::emit_expr;
#[cfg(test)]
pub(crate) use statements::emit_statement;
pub use statements::emit_statements;

const INDENT: &str = "    ";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decompiler::ast::*;

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
