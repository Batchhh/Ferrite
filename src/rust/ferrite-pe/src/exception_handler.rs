/// The kind of exception handler clause.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExceptionHandlerKind {
    Catch,
    Filter,
    Finally,
    Fault,
}

/// A single exception handler clause in a method body.
#[derive(Debug, Clone)]
pub struct ExceptionHandler {
    pub kind: ExceptionHandlerKind,
    pub try_offset: u32,
    pub try_length: u32,
    pub handler_offset: u32,
    pub handler_length: u32,
    pub class_token_or_filter: u32,
}
