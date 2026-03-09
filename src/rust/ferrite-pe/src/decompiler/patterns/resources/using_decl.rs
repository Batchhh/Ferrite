//! Using declaration pattern detection (C# 8).
//!
//! Detects `using var x = expr;` — scoped using declarations where the
//! compiler wraps all remaining statements in a try/finally Dispose block.
//! When a `Statement::Using` is the last statement in a block, it was likely
//! a using declaration. Rewrite to `UsingDecl` + flattened body.

use crate::decompiler::ast::*;

/// Detect using declarations and rewrite them.
pub fn detect_using_decl(stmts: Vec<Statement>) -> Vec<Statement> {
    let mut result: Vec<Statement> = stmts.into_iter().map(recurse_using_decl).collect();

    // If the last statement is Using(LocalDecl(..), body), flatten it
    flatten_trailing_using(&mut result);
    result
}

/// If the last statement is `Using(LocalDecl(..), body)`, replace with
/// `UsingDecl` followed by the flattened body statements (recursively).
fn flatten_trailing_using(stmts: &mut Vec<Statement>) {
    let should_flatten = matches!(
        stmts.last(),
        Some(Statement::Using(decl, _)) if matches!(decl.as_ref(), Statement::LocalDecl(_, _, Some(_)))
    );

    if !should_flatten {
        return;
    }

    let last = stmts.pop().unwrap();
    if let Statement::Using(decl, body) = last {
        if let Statement::LocalDecl(ty, name, Some(init)) = *decl {
            stmts.push(Statement::UsingDecl(ty, name, init));
            // Recursively process the body for chained using declarations
            let mut body = detect_using_decl(body);
            stmts.append(&mut body);
        }
    }
}

/// Recurse into all statement variants to detect nested using declarations.
fn recurse_using_decl(stmt: Statement) -> Statement {
    match stmt {
        Statement::If(c, tb, eb) => {
            Statement::If(c, detect_using_decl(tb), eb.map(detect_using_decl))
        }
        Statement::While(c, b) => Statement::While(c, detect_using_decl(b)),
        Statement::DoWhile(b, c) => Statement::DoWhile(detect_using_decl(b), c),
        Statement::For(i, c, u, b) => Statement::For(i, c, u, detect_using_decl(b)),
        Statement::ForEach(t, n, col, b) => Statement::ForEach(t, n, col, detect_using_decl(b)),
        Statement::Try(tb, catches, fb) => {
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_using_decl(c.body),
                    ..c
                })
                .collect();
            Statement::Try(detect_using_decl(tb), catches, fb.map(detect_using_decl))
        }
        Statement::Using(d, b) => Statement::Using(d, detect_using_decl(b)),
        Statement::Lock(e, b) => Statement::Lock(e, detect_using_decl(b)),
        Statement::Checked(b) => Statement::Checked(detect_using_decl(b)),
        Statement::Unchecked(b) => Statement::Unchecked(detect_using_decl(b)),
        Statement::Fixed(t, n, e, b) => Statement::Fixed(t, n, e, detect_using_decl(b)),
        Statement::Switch(e, cases, def) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_using_decl(b)))
                .collect();
            Statement::Switch(e, cases, def.map(detect_using_decl))
        }
        other => other,
    }
}
