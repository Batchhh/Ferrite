use super::super::compiler::is_compiler_generated_field;
use super::delegates_helpers::{expr_appears_in_stmt, substitute_in_stmt};
use crate::decompiler::ast::*;

pub fn propagate_delegate_assignments(stmts: Vec<Statement>) -> Vec<Statement> {
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
