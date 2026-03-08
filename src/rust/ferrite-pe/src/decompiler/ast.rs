//! C# AST types for the decompiler.
//!
//! These types represent a simplified C# abstract syntax tree, produced by
//! stack simulation and later refined by control-flow analysis.
#![allow(dead_code)]

/// An expression in the C# AST.
#[derive(Debug, Clone)]
pub enum Expr {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    This,
    /// Argument reference: (index, name).
    Arg(u16, String),
    /// Local variable reference: (index, type_name).
    Local(u16, String),
    /// Instance field access: obj.field.
    Field(Box<Expr>, String),
    /// Static field access: Type.field.
    StaticField(String, String),
    /// Array element access: arr[index].
    ArrayElement(Box<Expr>, Box<Expr>),
    /// Instance method call: obj?.method(args).
    Call(Option<Box<Expr>>, String, Vec<Expr>),
    /// Static method call: Type.method(args).
    StaticCall(String, String, Vec<Expr>),
    /// Constructor call: new Type(args).
    NewObj(String, Vec<Expr>),
    /// Binary operation.
    Binary(Box<Expr>, BinOp, Box<Expr>),
    /// Unary operation.
    Unary(UnaryOp, Box<Expr>),
    /// Type cast: (Type)expr.
    Cast(String, Box<Expr>),
    /// `is` type check.
    IsInst(Box<Expr>, String),
    /// `as` type check.
    AsInst(Box<Expr>, String),
    /// typeof(Type).
    Typeof(String),
    /// sizeof(Type).
    Sizeof(String),
    /// default(Type).
    Default(String),
    /// new Type[size].
    ArrayNew(String, Box<Expr>),
    /// new Type[] { elem0, elem1, ... }.
    ArrayInit(String, Vec<Expr>),
    /// arr.Length.
    ArrayLength(Box<Expr>),
    /// Boxing: (object)value.
    Box(String, Box<Expr>),
    /// Unboxing.
    Unbox(String, Box<Expr>),
    /// Address-of (&variable).
    AddressOf(Box<Expr>),
    /// Ternary expression: condition ? then : else.
    Ternary(Box<Expr>, Box<Expr>, Box<Expr>),
    /// Lambda expression: params [(type, name)], body (single expr or block).
    Lambda(Vec<(String, String)>, Box<LambdaBody>),
    /// Fallback for unhandled IL.
    Raw(String),
}

/// The body of a lambda expression.
#[derive(Debug, Clone)]
pub enum LambdaBody {
    /// Expression-bodied lambda: (params) => expr
    Expr(Expr),
    /// Block-bodied lambda: (params) => { statements }
    Block(Vec<Statement>),
}

/// Binary operators.
#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    LogicalAnd,
    LogicalOr,
    NullCoalesce,
}

/// Unary operators.
#[derive(Debug, Clone)]
pub enum UnaryOp {
    Neg,
    Not,
    LogicalNot,
}

/// A C# statement.
#[derive(Debug, Clone)]
pub enum Statement {
    /// Expression statement.
    Expr(Expr),
    /// Return statement with optional value.
    Return(Option<Expr>),
    /// Assignment: lhs = rhs.
    Assign(Expr, Expr),
    /// If/else.
    If(Expr, Block, Option<Block>),
    /// While loop.
    While(Expr, Block),
    /// Do-while loop.
    DoWhile(Block, Expr),
    /// For loop: (init, condition, increment, body).
    For(Box<Statement>, Expr, Box<Statement>, Block),
    /// Foreach loop: (type, var_name, collection, body).
    ForEach(String, String, Expr, Block),
    /// Switch statement: (expr, cases, default).
    Switch(Expr, Vec<(Vec<Expr>, Block)>, Option<Block>),
    /// Try/catch/finally.
    Try(Block, Vec<CatchClause>, Option<Block>),
    /// Throw statement.
    Throw(Option<Expr>),
    /// Break.
    Break,
    /// Continue.
    Continue,
    /// Using statement.
    Using(Box<Statement>, Block),
    /// Local variable declaration: (type, name, initializer).
    LocalDecl(String, String, Option<Expr>),
}

/// A catch clause in a try/catch.
#[derive(Debug, Clone)]
pub struct CatchClause {
    pub exception_type: String,
    pub var_name: Option<String>,
    pub body: Block,
}

/// A block of statements.
pub type Block = Vec<Statement>;
