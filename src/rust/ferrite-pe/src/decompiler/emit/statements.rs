//! Statement emitters — converts AST statement nodes to C# source strings.

use crate::decompiler::ast::*;

use super::expressions::emit_expr;
use super::INDENT;

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
            emit_try(&prefix, try_block, catches, finally_block, indent)
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
        Statement::Lock(expr, body) => {
            let mut s = format!("{}lock ({})\n{}{{\n", prefix, emit_expr(expr), prefix);
            s += &emit_statements(body, indent + 1);
            s += &format!("{}}}\n", prefix);
            s
        }
        Statement::Checked(body) => {
            let mut s = format!("{}checked\n{}{{\n", prefix, prefix);
            s += &emit_statements(body, indent + 1);
            s += &format!("{}}}\n", prefix);
            s
        }
        Statement::Unchecked(body) => {
            let mut s = format!("{}unchecked\n{}{{\n", prefix, prefix);
            s += &emit_statements(body, indent + 1);
            s += &format!("{}}}\n", prefix);
            s
        }
        Statement::UsingDecl(ty, name, init) => {
            format!("{}using {} {} = {};\n", prefix, ty, name, emit_expr(init))
        }
        Statement::Fixed(ty, name, expr, body) => {
            let mut s = format!(
                "{}fixed ({} {} = {})\n{}{{\n",
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
        Statement::YieldReturn(expr) => {
            format!("{}yield return {};\n", prefix, emit_expr(expr))
        }
        Statement::YieldBreak => format!("{}yield break;\n", prefix),
        Statement::TupleDeconstruct(vars, expr) => {
            let vars_str = vars
                .iter()
                .map(|(ty, name)| format!("{} {}", ty, name))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}var ({}) = {};\n", prefix, vars_str, emit_expr(expr))
        }
    }
}

fn emit_try(
    prefix: &str,
    try_block: &[Statement],
    catches: &[CatchClause],
    finally_block: &Option<Block>,
    indent: usize,
) -> String {
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
