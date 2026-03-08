//! Expression emitters — converts AST expression nodes to C# source strings.

use crate::decompiler::ast::{BinOp, Expr, LambdaBody, UnaryOp};

use super::emit_statements;

pub fn emit_expr(expr: &Expr) -> String {
    match expr {
        Expr::Null => "null".into(),
        Expr::Bool(true) => "true".into(),
        Expr::Bool(false) => "false".into(),
        Expr::Int(v) => v.to_string(),
        Expr::Float(v) => format_float(*v),
        Expr::String(s) => format!("\"{}\"", escape_string(s)),
        Expr::This => "this".into(),
        Expr::Arg(_, name) => name.clone(),
        Expr::Local(_, name) => name.clone(),
        Expr::Field(obj, name) => {
            if matches!(obj.as_ref(), Expr::This) {
                name.clone()
            } else {
                format!("{}.{}", emit_expr(obj), name)
            }
        }
        Expr::StaticField(ty, name) => {
            if ty.is_empty() {
                name.clone()
            } else {
                format!("{}.{}", ty, name)
            }
        }
        Expr::ArrayElement(arr, idx) => format!("{}[{}]", emit_expr(arr), emit_expr(idx)),
        Expr::Call(obj, name, args) => {
            let args_str = args.iter().map(emit_expr).collect::<Vec<_>>().join(", ");
            if let Some(o) = obj {
                if matches!(o.as_ref(), Expr::This) {
                    format!("{}({})", name, args_str)
                } else {
                    format!("{}.{}({})", emit_expr(o), name, args_str)
                }
            } else {
                format!("{}({})", name, args_str)
            }
        }
        Expr::StaticCall(ty, name, args) => {
            let args_str = args.iter().map(emit_expr).collect::<Vec<_>>().join(", ");
            if ty.is_empty() {
                format!("{}({})", name, args_str)
            } else {
                format!("{}.{}({})", ty, name, args_str)
            }
        }
        Expr::NewObj(ty, args) => {
            let args_str = args.iter().map(emit_expr).collect::<Vec<_>>().join(", ");
            format!("new {}({})", ty, args_str)
        }
        Expr::Binary(left, op, right) => {
            format!(
                "{} {} {}",
                emit_expr_with_parens(left, op),
                emit_binop(op),
                emit_expr_with_parens(right, op)
            )
        }
        Expr::Unary(op, expr) => format!("{}{}", emit_unary_op(op), emit_expr(expr)),
        Expr::Cast(ty, expr) => format!("({}){}", ty, emit_expr(expr)),
        Expr::IsInst(expr, ty) => format!("{} is {}", emit_expr(expr), ty),
        Expr::AsInst(expr, ty) => format!("{} as {}", emit_expr(expr), ty),
        Expr::Typeof(ty) => format!("typeof({})", ty),
        Expr::Sizeof(ty) => format!("sizeof({})", ty),
        Expr::Default(ty) => format!("default({})", ty),
        Expr::ArrayNew(ty, size) => format!("new {}[{}]", ty, emit_expr(size)),
        Expr::ArrayInit(ty, elems) => {
            let elems_str = elems.iter().map(emit_expr).collect::<Vec<_>>().join(", ");
            format!("new {}[] {{ {} }}", ty, elems_str)
        }
        Expr::ArrayLength(arr) => format!("{}.Length", emit_expr(arr)),
        Expr::Box(_, expr) | Expr::Unbox(_, expr) => emit_expr(expr),
        Expr::AddressOf(expr) => format!("ref {}", emit_expr(expr)),
        Expr::Ternary(cond, then_val, else_val) => {
            format!(
                "{} ? {} : {}",
                emit_expr(cond),
                emit_expr(then_val),
                emit_expr(else_val)
            )
        }
        Expr::Lambda(params, body) => {
            let params_str = params
                .iter()
                .map(|(ty, name)| format!("{} {}", ty, name))
                .collect::<Vec<_>>()
                .join(", ");
            match body.as_ref() {
                LambdaBody::Expr(expr) => format!("({}) => {}", params_str, emit_expr(expr)),
                LambdaBody::Block(stmts) => {
                    let body_str = emit_statements(stmts, 0);
                    format!("({}) => {{\n{}}}", params_str, body_str)
                }
            }
        }
        Expr::Raw(s) => s.clone(),
    }
}

fn emit_expr_with_parens(expr: &Expr, _parent_op: &BinOp) -> String {
    // Add parentheses for nested binary expressions to ensure correctness.
    match expr {
        Expr::Binary(_, _, _) => format!("({})", emit_expr(expr)),
        _ => emit_expr(expr),
    }
}

fn emit_binop(op: &BinOp) -> &str {
    match op {
        BinOp::Add => "+",
        BinOp::Sub => "-",
        BinOp::Mul => "*",
        BinOp::Div => "/",
        BinOp::Rem => "%",
        BinOp::And => "&",
        BinOp::Or => "|",
        BinOp::Xor => "^",
        BinOp::Shl => "<<",
        BinOp::Shr => ">>",
        BinOp::Eq => "==",
        BinOp::Ne => "!=",
        BinOp::Lt => "<",
        BinOp::Gt => ">",
        BinOp::Le => "<=",
        BinOp::Ge => ">=",
        BinOp::LogicalAnd => "&&",
        BinOp::LogicalOr => "||",
        BinOp::NullCoalesce => "??",
    }
}

fn emit_unary_op(op: &UnaryOp) -> &str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::Not => "~",
        UnaryOp::LogicalNot => "!",
    }
}

fn format_float(v: f64) -> String {
    if v == v.floor() && v.abs() < 1e15 {
        format!("{:.1}f", v)
    } else {
        format!("{}f", v)
    }
}

fn escape_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
        .replace('\0', "\\0")
}
