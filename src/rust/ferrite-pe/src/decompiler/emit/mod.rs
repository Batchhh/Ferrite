//! C# code emitter — converts AST nodes to formatted C# source strings.

pub mod expressions;
mod format;
pub mod statements;

pub use expressions::emit_expr;
pub use statements::emit_statements;

const INDENT: &str = "    ";
