use crate::decompiler::ast::*;

pub(super) fn clean_up(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result: Vec<Statement> = Vec::with_capacity(stmts.len());

    for stmt in stmts {
        let stmt = recurse_cleanup(stmt);

        if let Statement::If(_, ref then_block, ref else_block) = stmt {
            if then_block.is_empty() && else_block.is_none() {
                continue;
            }
        }

        if let Statement::Expr(Expr::Raw(ref s)) = stmt {
            if s.contains("nop") || s.is_empty() {
                continue;
            }
        }

        result.push(stmt);
    }

    if let Some(Statement::Return(None)) = result.last() {
        if result.len() > 1 {
            result.pop();
        }
    }

    result
}

fn recurse_cleanup(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(cond, then_block, else_block) => {
            Statement::If(cond, clean_up(then_block), else_block.map(clean_up))
        }
        Statement::While(cond, body) => Statement::While(cond, clean_up(body)),
        Statement::DoWhile(body, cond) => Statement::DoWhile(clean_up(body), cond),
        Statement::For(init, cond, update, body) => {
            Statement::For(init, cond, update, clean_up(body))
        }
        Statement::ForEach(ty, name, coll, body) => {
            Statement::ForEach(ty, name, coll, clean_up(body))
        }
        Statement::Try(try_body, catches, finally_block) => {
            let try_body = clean_up(try_body);
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: clean_up(c.body),
                    ..c
                })
                .collect();
            let finally_block = finally_block.map(clean_up);
            Statement::Try(try_body, catches, finally_block)
        }
        Statement::Using(decl, body) => Statement::Using(decl, clean_up(body)),
        Statement::Switch(expr, cases, default) => {
            let cases = cases.into_iter().map(|(l, b)| (l, clean_up(b))).collect();
            Statement::Switch(expr, cases, default.map(clean_up))
        }
        other => other,
    }
}
