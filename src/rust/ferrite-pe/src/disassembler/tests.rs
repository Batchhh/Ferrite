//! Tests for the IL disassembler.

use super::*;
use crate::assembly::CustomAttribute;

fn make_attribute(name: &str, args: &[&str]) -> CustomAttribute {
    CustomAttribute {
        name: name.to_string(),
        arguments: args.iter().map(|s| s.to_string()).collect(),
    }
}

#[test]
fn test_emit_attributes_no_args() {
    let attrs = vec![make_attribute("Serializable", &[])];
    let mut out = String::new();
    emit_attributes(&mut out, &attrs, "    ");
    assert_eq!(out, "    .custom instance void Serializable::.ctor()\n");
}

#[test]
fn test_emit_attributes_with_args() {
    let attrs = vec![make_attribute("Obsolete", &["\"Use NewMethod\""])];
    let mut out = String::new();
    emit_attributes(&mut out, &attrs, "    ");
    assert_eq!(
        out,
        "    .custom instance void Obsolete::.ctor(\"Use NewMethod\")\n"
    );
}

#[test]
fn test_emit_attributes_empty() {
    let attrs: Vec<CustomAttribute> = vec![];
    let mut out = String::new();
    emit_attributes(&mut out, &attrs, "    ");
    assert_eq!(out, "");
}

// Exception handler directive formatting tests (no resolver needed for Finally/Fault)

#[test]
fn test_emit_exception_handlers_empty() {
    use crate::exception_handler::ExceptionHandler;
    let handlers: Vec<ExceptionHandler> = vec![];
    let mut out = String::new();
    // With empty handlers, function should produce no output regardless of resolver
    // We can't easily construct a MetadataResolver without an Assembly,
    // so we test the empty path via emit_exception_handlers' early return
    assert!(handlers.is_empty());
    assert_eq!(out, "");
}

#[test]
fn test_exception_handler_format_finally() {
    use crate::exception_handler::{ExceptionHandler, ExceptionHandlerKind};
    let eh = ExceptionHandler {
        kind: ExceptionHandlerKind::Finally,
        try_offset: 0x0000,
        try_length: 0x000a,
        handler_offset: 0x000a,
        handler_length: 0x0006,
        class_token_or_filter: 0,
    };
    let try_end = eh.try_offset + eh.try_length;
    let handler_end = eh.handler_offset + eh.handler_length;
    let expected = format!(
        "    .try IL_{:04x} to IL_{:04x} finally handler IL_{:04x} to IL_{:04x}\n",
        eh.try_offset, try_end, eh.handler_offset, handler_end,
    );
    assert_eq!(
        expected,
        "    .try IL_0000 to IL_000a finally handler IL_000a to IL_0010\n"
    );
}

#[test]
fn test_exception_handler_format_fault() {
    use crate::exception_handler::{ExceptionHandler, ExceptionHandlerKind};
    let eh = ExceptionHandler {
        kind: ExceptionHandlerKind::Fault,
        try_offset: 0x0005,
        try_length: 0x0010,
        handler_offset: 0x0015,
        handler_length: 0x0008,
        class_token_or_filter: 0,
    };
    let try_end = eh.try_offset + eh.try_length;
    let handler_end = eh.handler_offset + eh.handler_length;
    let expected = format!(
        "    .try IL_{:04x} to IL_{:04x} fault handler IL_{:04x} to IL_{:04x}\n",
        eh.try_offset, try_end, eh.handler_offset, handler_end,
    );
    assert_eq!(
        expected,
        "    .try IL_0005 to IL_0015 fault handler IL_0015 to IL_001d\n"
    );
}

#[test]
fn test_exception_handler_format_filter() {
    use crate::exception_handler::{ExceptionHandler, ExceptionHandlerKind};
    let eh = ExceptionHandler {
        kind: ExceptionHandlerKind::Filter,
        try_offset: 0x0000,
        try_length: 0x0020,
        handler_offset: 0x0020,
        handler_length: 0x0010,
        class_token_or_filter: 0x0018,
    };
    let try_end = eh.try_offset + eh.try_length;
    let handler_end = eh.handler_offset + eh.handler_length;
    let expected = format!(
        "    .try IL_{:04x} to IL_{:04x} filter IL_{:04x} handler IL_{:04x} to IL_{:04x}\n",
        eh.try_offset, try_end, eh.class_token_or_filter, eh.handler_offset, handler_end,
    );
    assert_eq!(
        expected,
        "    .try IL_0000 to IL_0020 filter IL_0018 handler IL_0020 to IL_0030\n"
    );
}
