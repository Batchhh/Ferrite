//! Range and Index operator pattern detection.
//!
//! Detects `new Index(n, true)` → `^n` and `new Range(start, end)` → `start..end`.

use crate::decompiler::ast::*;

/// Detect range/index patterns and rewrite them.
pub fn detect_range_index(stmts: Vec<Statement>) -> Vec<Statement> {
    stmts.into_iter().map(transform_stmt).collect()
}

/// Transform expressions within a statement.
fn transform_stmt(stmt: Statement) -> Statement {
    match stmt {
        Statement::Expr(e) => Statement::Expr(transform_expr(e)),
        Statement::Return(e) => Statement::Return(e.map(transform_expr)),
        Statement::Assign(t, v) => Statement::Assign(transform_expr(t), transform_expr(v)),
        Statement::LocalDecl(ty, name, init) => {
            Statement::LocalDecl(ty, name, init.map(transform_expr))
        }
        Statement::If(c, tb, eb) => Statement::If(
            transform_expr(c),
            detect_range_index(tb),
            eb.map(detect_range_index),
        ),
        Statement::While(c, b) => Statement::While(transform_expr(c), detect_range_index(b)),
        Statement::DoWhile(b, c) => Statement::DoWhile(detect_range_index(b), transform_expr(c)),
        Statement::For(i, c, u, b) => Statement::For(
            Box::new(transform_stmt(*i)),
            transform_expr(c),
            Box::new(transform_stmt(*u)),
            detect_range_index(b),
        ),
        Statement::ForEach(t, n, col, b) => {
            Statement::ForEach(t, n, transform_expr(col), detect_range_index(b))
        }
        Statement::Try(tb, catches, fb) => {
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_range_index(c.body),
                    ..c
                })
                .collect();
            Statement::Try(detect_range_index(tb), catches, fb.map(detect_range_index))
        }
        Statement::Using(d, b) => Statement::Using(d, detect_range_index(b)),
        Statement::Lock(e, b) => Statement::Lock(e, detect_range_index(b)),
        Statement::Checked(b) => Statement::Checked(detect_range_index(b)),
        Statement::Unchecked(b) => Statement::Unchecked(detect_range_index(b)),
        Statement::Fixed(t, n, e, b) => {
            Statement::Fixed(t, n, transform_expr(e), detect_range_index(b))
        }
        Statement::Switch(e, cases, def) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_range_index(b)))
                .collect();
            Statement::Switch(transform_expr(e), cases, def.map(detect_range_index))
        }
        Statement::Throw(e) => Statement::Throw(e.map(transform_expr)),
        Statement::UsingDecl(t, n, e) => Statement::UsingDecl(t, n, transform_expr(e)),
        Statement::YieldReturn(e) => Statement::YieldReturn(transform_expr(e)),
        Statement::TupleDeconstruct(vars, e) => {
            Statement::TupleDeconstruct(vars, transform_expr(e))
        }
        other => other,
    }
}

/// Recursively transform expressions, rewriting Index/Range constructors.
fn transform_expr(expr: Expr) -> Expr {
    match expr {
        // Index from end: new Index(n, true) → ^n
        Expr::NewObj(ref ty, ref args) if ty == "Index" && args.len() == 2 => match &args[1] {
            Expr::Bool(true) => Expr::IndexFromEnd(Box::new(transform_expr(args[0].clone()))),
            Expr::Bool(false) => transform_expr(args[0].clone()),
            _ => recurse_expr(expr),
        },
        // Range: new Range(start, end) → start..end
        Expr::NewObj(ref ty, ref args) if ty == "Range" && args.len() == 2 => Expr::RangeExpr(
            Some(Box::new(transform_expr(args[0].clone()))),
            Some(Box::new(transform_expr(args[1].clone()))),
        ),
        other => recurse_expr(other),
    }
}

/// Recurse into sub-expressions.
fn recurse_expr(expr: Expr) -> Expr {
    match expr {
        Expr::Field(o, n) => Expr::Field(Box::new(transform_expr(*o)), n),
        Expr::ArrayElement(a, i) => {
            Expr::ArrayElement(Box::new(transform_expr(*a)), Box::new(transform_expr(*i)))
        }
        Expr::Call(obj, name, args) => Expr::Call(
            obj.map(|o| Box::new(transform_expr(*o))),
            name,
            args.into_iter().map(transform_expr).collect(),
        ),
        Expr::StaticCall(t, n, args) => {
            Expr::StaticCall(t, n, args.into_iter().map(transform_expr).collect())
        }
        Expr::NewObj(t, args) => Expr::NewObj(t, args.into_iter().map(transform_expr).collect()),
        Expr::Binary(l, op, r) => Expr::Binary(
            Box::new(transform_expr(*l)),
            op,
            Box::new(transform_expr(*r)),
        ),
        Expr::Unary(op, e) => Expr::Unary(op, Box::new(transform_expr(*e))),
        Expr::Cast(t, e) => Expr::Cast(t, Box::new(transform_expr(*e))),
        Expr::Box(t, e) => Expr::Box(t, Box::new(transform_expr(*e))),
        Expr::Unbox(t, e) => Expr::Unbox(t, Box::new(transform_expr(*e))),
        Expr::Ternary(c, t, e) => Expr::Ternary(
            Box::new(transform_expr(*c)),
            Box::new(transform_expr(*t)),
            Box::new(transform_expr(*e)),
        ),
        Expr::AddressOf(e) => Expr::AddressOf(Box::new(transform_expr(*e))),
        Expr::IsInst(e, t) => Expr::IsInst(Box::new(transform_expr(*e)), t),
        Expr::AsInst(e, t) => Expr::AsInst(Box::new(transform_expr(*e)), t),
        Expr::ArrayNew(t, sz) => Expr::ArrayNew(t, Box::new(transform_expr(*sz))),
        Expr::ArrayInit(t, elems) => {
            Expr::ArrayInit(t, elems.into_iter().map(transform_expr).collect())
        }
        Expr::ArrayLength(e) => Expr::ArrayLength(Box::new(transform_expr(*e))),
        Expr::IndexFromEnd(e) => Expr::IndexFromEnd(Box::new(transform_expr(*e))),
        Expr::TupleExpr(elems) => Expr::TupleExpr(elems.into_iter().map(transform_expr).collect()),
        Expr::Await(e) => Expr::Await(Box::new(transform_expr(*e))),
        Expr::Stackalloc(t, sz) => Expr::Stackalloc(t, Box::new(transform_expr(*sz))),
        other => other,
    }
}
