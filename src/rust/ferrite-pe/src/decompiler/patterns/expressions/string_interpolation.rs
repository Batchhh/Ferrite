//! String interpolation pattern detection.
//!
//! Rewrites `String.Format("..{0}..", a)` → `$"..{a}.."` and
//! `String.Concat(a, b)` → `$"{a}{b}"`.

use crate::decompiler::ast::*;

pub fn detect_string_interpolation(stmts: Vec<Statement>) -> Vec<Statement> {
    stmts.into_iter().map(transform_stmt).collect()
}

fn transform_stmt(stmt: Statement) -> Statement {
    match stmt {
        Statement::Expr(e) => Statement::Expr(transform_expr(e)),
        Statement::Return(Some(e)) => Statement::Return(Some(transform_expr(e))),
        Statement::Assign(t, v) => Statement::Assign(transform_expr(t), transform_expr(v)),
        Statement::LocalDecl(ty, n, init) => Statement::LocalDecl(ty, n, init.map(transform_expr)),
        Statement::If(c, tb, eb) => Statement::If(
            transform_expr(c),
            detect_string_interpolation(tb),
            eb.map(detect_string_interpolation),
        ),
        Statement::While(c, b) => {
            Statement::While(transform_expr(c), detect_string_interpolation(b))
        }
        Statement::DoWhile(b, c) => {
            Statement::DoWhile(detect_string_interpolation(b), transform_expr(c))
        }
        Statement::For(i, c, u, b) => Statement::For(
            Box::new(transform_stmt(*i)),
            transform_expr(c),
            Box::new(transform_stmt(*u)),
            detect_string_interpolation(b),
        ),
        Statement::ForEach(t, n, col, b) => {
            Statement::ForEach(t, n, transform_expr(col), detect_string_interpolation(b))
        }
        Statement::Try(tb, catches, fb) => {
            let catches = catches
                .into_iter()
                .map(|c| CatchClause {
                    body: detect_string_interpolation(c.body),
                    ..c
                })
                .collect();
            Statement::Try(
                detect_string_interpolation(tb),
                catches,
                fb.map(detect_string_interpolation),
            )
        }
        Statement::Switch(e, cases, def) => {
            let cases = cases
                .into_iter()
                .map(|(l, b)| (l, detect_string_interpolation(b)))
                .collect();
            Statement::Switch(
                transform_expr(e),
                cases,
                def.map(detect_string_interpolation),
            )
        }
        Statement::Using(d, b) => Statement::Using(d, detect_string_interpolation(b)),
        Statement::Lock(e, b) => Statement::Lock(e, detect_string_interpolation(b)),
        Statement::Checked(b) => Statement::Checked(detect_string_interpolation(b)),
        Statement::Unchecked(b) => Statement::Unchecked(detect_string_interpolation(b)),
        Statement::Fixed(t, n, e, b) => {
            Statement::Fixed(t, n, transform_expr(e), detect_string_interpolation(b))
        }
        Statement::Throw(Some(e)) => Statement::Throw(Some(transform_expr(e))),
        Statement::UsingDecl(t, n, e) => Statement::UsingDecl(t, n, transform_expr(e)),
        Statement::YieldReturn(e) => Statement::YieldReturn(transform_expr(e)),
        Statement::TupleDeconstruct(vars, e) => {
            Statement::TupleDeconstruct(vars, transform_expr(e))
        }
        other => other,
    }
}

fn transform_expr(expr: Expr) -> Expr {
    let expr = recurse_expr(expr);
    match &expr {
        Expr::StaticCall(ty, method, args) if ty == "String" || ty == "System.String" => {
            match method.as_str() {
                "Format" => try_format(args).unwrap_or(expr),
                "Concat" if args.len() >= 2 => build_concat(args),
                _ => expr,
            }
        }
        _ => expr,
    }
}

fn recurse_expr(expr: Expr) -> Expr {
    match expr {
        Expr::Call(obj, n, args) => {
            let obj = obj.map(|o| Box::new(transform_expr(*o)));
            let args = args.into_iter().map(transform_expr).collect();
            Expr::Call(obj, n, args)
        }
        Expr::StaticCall(t, n, args) => {
            let args = args.into_iter().map(transform_expr).collect();
            Expr::StaticCall(t, n, args)
        }
        Expr::NewObj(t, args) => Expr::NewObj(t, args.into_iter().map(transform_expr).collect()),
        Expr::Binary(l, op, r) => Expr::Binary(
            Box::new(transform_expr(*l)),
            op,
            Box::new(transform_expr(*r)),
        ),
        Expr::Unary(op, e) => Expr::Unary(op, Box::new(transform_expr(*e))),
        Expr::Ternary(c, t, e) => Expr::Ternary(
            Box::new(transform_expr(*c)),
            Box::new(transform_expr(*t)),
            Box::new(transform_expr(*e)),
        ),
        Expr::Field(o, n) => Expr::Field(Box::new(transform_expr(*o)), n),
        Expr::Cast(t, e) => Expr::Cast(t, Box::new(transform_expr(*e))),
        Expr::ArrayElement(a, i) => {
            Expr::ArrayElement(Box::new(transform_expr(*a)), Box::new(transform_expr(*i)))
        }
        other => other,
    }
}

/// `String.Format("Hello {0}", name)` → interpolated string
fn try_format(args: &[Expr]) -> Option<Expr> {
    let fmt = match args.first()? {
        Expr::String(s) => s,
        _ => return None,
    };
    let value_args = &args[1..];
    let mut parts = Vec::new();
    let mut rest = fmt.as_str();
    while let Some(open) = rest.find('{') {
        if open > 0 {
            parts.push(InterpolatedPart::Literal(rest[..open].to_string()));
        }
        let close = rest[open..].find('}')? + open;
        let idx: usize = rest[open + 1..close].parse().ok()?;
        if idx >= value_args.len() {
            return None;
        }
        parts.push(InterpolatedPart::Expression(value_args[idx].clone()));
        rest = &rest[close + 1..];
    }
    if !rest.is_empty() {
        parts.push(InterpolatedPart::Literal(rest.to_string()));
    }
    Some(Expr::InterpolatedString(parts))
}

/// `String.Concat(a, b, c)` → interpolated string
fn build_concat(args: &[Expr]) -> Expr {
    let parts = args
        .iter()
        .map(|a| match a {
            Expr::String(s) => InterpolatedPart::Literal(s.clone()),
            other => InterpolatedPart::Expression(other.clone()),
        })
        .collect();
    Expr::InterpolatedString(parts)
}
