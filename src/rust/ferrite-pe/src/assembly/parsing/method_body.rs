use dotnetdll::prelude::*;

use crate::exception_handler::{ExceptionHandler, ExceptionHandlerKind};

use super::super::formatting::format_types::format_method_type;
use super::super::{Assembly, ParsedMethodBody};
use super::instructions::convert_instruction;

/// Convert a dotnetdll method body to [`ParsedMethodBody`].
pub(in crate::assembly) fn convert_method_body(
    body: &dotnetdll::resolved::body::Method,
    res: &Resolution,
    asm: &mut Assembly,
) -> ParsedMethodBody {
    let mut instructions = Vec::with_capacity(body.instructions.len());

    for (idx, instr) in body.instructions.iter().enumerate() {
        let offset = idx as u32;
        let (opcode, operand) = convert_instruction(instr, res, asm);
        instructions.push(crate::il::Instruction {
            offset,
            opcode,
            operand,
        });
    }

    let exception_handlers = convert_exception_handlers(&body.data_sections);

    let locals: Vec<String> = body
        .header
        .local_variables
        .iter()
        .map(|lv| format_local_variable(lv, res))
        .collect();

    ParsedMethodBody {
        instructions,
        exception_handlers,
        locals,
        max_stack: body.header.maximum_stack_size as u16,
    }
}

/// Format a local variable's type name for the decompiler.
fn format_local_variable(
    lv: &dotnetdll::resolved::types::LocalVariable,
    res: &Resolution,
) -> String {
    match lv {
        dotnetdll::resolved::types::LocalVariable::TypedReference => "TypedReference".into(),
        dotnetdll::resolved::types::LocalVariable::Variable { var_type, .. } => {
            format_method_type(var_type, res)
        }
    }
}

/// Convert dotnetdll exception handler sections to [`ExceptionHandler`] records.
fn convert_exception_handlers(
    sections: &[dotnetdll::resolved::body::DataSection],
) -> Vec<ExceptionHandler> {
    let mut handlers = Vec::new();
    for section in sections {
        if let dotnetdll::resolved::body::DataSection::ExceptionHandlers(exceptions) = section {
            for ex in exceptions {
                let (kind, class_token_or_filter) = match &ex.kind {
                    dotnetdll::resolved::body::ExceptionKind::TypedException(_) => {
                        (ExceptionHandlerKind::Catch, 0)
                    }
                    dotnetdll::resolved::body::ExceptionKind::Filter { offset } => {
                        (ExceptionHandlerKind::Filter, *offset as u32)
                    }
                    dotnetdll::resolved::body::ExceptionKind::Finally => {
                        (ExceptionHandlerKind::Finally, 0)
                    }
                    dotnetdll::resolved::body::ExceptionKind::Fault => {
                        (ExceptionHandlerKind::Fault, 0)
                    }
                };
                handlers.push(ExceptionHandler {
                    kind,
                    try_offset: ex.try_offset as u32,
                    try_length: ex.try_length as u32,
                    handler_offset: ex.handler_offset as u32,
                    handler_length: ex.handler_length as u32,
                    class_token_or_filter,
                });
            }
        }
    }
    handlers
}
