//! Emit ildasm-style exception handler directives.

use crate::decompiler::resolver::MetadataResolver;
use crate::exception_handler::{ExceptionHandler, ExceptionHandlerKind};

/// Appends `.try … catch/finally/fault/filter handler …` directives for each exception clause.
pub fn emit_exception_handlers(
    out: &mut String,
    handlers: &[ExceptionHandler],
    resolver: &MetadataResolver,
    indent: &str,
) {
    if handlers.is_empty() {
        return;
    }
    for eh in handlers {
        let try_end = eh.try_offset + eh.try_length;
        let handler_end = eh.handler_offset + eh.handler_length;
        let handler_desc = match eh.kind {
            ExceptionHandlerKind::Catch => {
                let class_name = resolver.resolve_token(eh.class_token_or_filter);
                format!("catch {class_name}")
            }
            ExceptionHandlerKind::Finally => "finally".to_string(),
            ExceptionHandlerKind::Fault => "fault".to_string(),
            ExceptionHandlerKind::Filter => {
                format!("filter IL_{:04x}", eh.class_token_or_filter)
            }
        };
        out.push_str(&format!(
            "{indent}.try IL_{:04x} to IL_{:04x} {handler_desc} handler IL_{:04x} to IL_{:04x}\n",
            eh.try_offset, try_end, eh.handler_offset, handler_end,
        ));
    }
    out.push('\n');
}
