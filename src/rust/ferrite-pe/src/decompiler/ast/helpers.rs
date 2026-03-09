//! Helper types for the C# AST.

use super::{Expr, Statement};

/// A part of an interpolated string.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum InterpolatedPart {
    Literal(String),
    Expression(Expr),
}

/// A catch clause in a try/catch.
#[derive(Debug, Clone)]
pub struct CatchClause {
    pub exception_type: String,
    pub var_name: Option<String>,
    pub body: Block,
}

/// The body of a lambda expression.
#[derive(Debug, Clone)]
pub enum LambdaBody {
    /// Expression-bodied lambda: (params) => expr
    Expr(Expr),
    /// Block-bodied lambda: (params) => { statements }
    Block(Vec<Statement>),
}

/// A block of statements.
pub type Block = Vec<Statement>;
