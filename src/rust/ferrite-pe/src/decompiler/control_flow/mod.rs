//! Control flow analysis — converts IL branch patterns into structured statements.
//!
//! This module uses a pattern-based approach to detect if/else, loops, switch,
//! and try/catch/finally structures directly from the IL instruction stream,
//! without building a full CFG.
//!
//! Key architectural decision: `process_region_impl` threads a single
//! `StackSimulator` through each region so stack values persist across
//! linear blocks and ternary expressions are correctly detected.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

use crate::decompiler::ast::*;
use crate::decompiler::resolver::MetadataResolver;
use crate::decompiler::stack::StackSimulator;
use crate::exception_handler::{ExceptionHandler, ExceptionHandlerKind};
use crate::il::{Instruction, OpCode, Operand};

mod analysis;
mod exceptions;
mod reconstruction;

/// Analyzes IL instruction streams and produces structured C# statements.
pub struct ControlFlowAnalyzer<'a> {
    pub(super) resolver: &'a MetadataResolver<'a>,
    pub(super) instructions: &'a [Instruction],
    pub(super) exception_handlers: &'a [ExceptionHandler],
    pub(super) locals: Vec<String>,
    pub(super) params: Vec<String>,
    pub(super) is_static: bool,
    /// Maps byte offset -> instruction index for quick lookup.
    pub(super) offset_to_index: HashMap<u32, usize>,
    /// Tracks which exception handler indices have already been consumed.
    pub(super) consumed_handlers: RefCell<HashSet<usize>>,
}

impl<'a> ControlFlowAnalyzer<'a> {
    pub fn new(
        resolver: &'a MetadataResolver<'a>,
        instructions: &'a [Instruction],
        exception_handlers: &'a [ExceptionHandler],
        locals: Vec<String>,
        params: Vec<String>,
        is_static: bool,
    ) -> Self {
        let mut offset_to_index = HashMap::new();
        for (i, instr) in instructions.iter().enumerate() {
            offset_to_index.insert(instr.offset, i);
        }
        Self {
            resolver,
            instructions,
            exception_handlers,
            locals,
            params,
            is_static,
            offset_to_index,
            consumed_handlers: RefCell::new(HashSet::new()),
        }
    }

    /// Maximum recursion depth to prevent stack overflow on complex/malformed IL.
    pub(super) const MAX_DEPTH: usize = 64;

    /// Main entry point: analyze all instructions and produce structured statements.
    pub fn analyze(&self) -> Vec<Statement> {
        self.process_region_impl(0, self.instructions.len(), 0)
    }
}

pub(super) struct StructuredResult {
    pub(super) stmt: Statement,
    pub(super) next_idx: usize,
}

pub(super) struct LoopResult {
    pub(super) stmts: Vec<Statement>,
    pub(super) next_idx: usize,
}

/// Result from if/else analysis — either a normal statement or a ternary
/// value pushed onto the shared simulator's stack.
pub(super) enum FlowResult {
    Statement(Statement, usize),
    TernaryPushed(usize),
}

pub(super) fn is_conditional_branch(opcode: OpCode) -> bool {
    matches!(
        opcode,
        OpCode::Brfalse
            | OpCode::BrfalseS
            | OpCode::Brtrue
            | OpCode::BrtrueS
            | OpCode::Beq
            | OpCode::BeqS
            | OpCode::Bge
            | OpCode::BgeS
            | OpCode::BgeUn
            | OpCode::BgeUnS
            | OpCode::Bgt
            | OpCode::BgtS
            | OpCode::BgtUn
            | OpCode::BgtUnS
            | OpCode::Ble
            | OpCode::BleS
            | OpCode::BleUn
            | OpCode::BleUnS
            | OpCode::Blt
            | OpCode::BltS
            | OpCode::BltUn
            | OpCode::BltUnS
            | OpCode::BneUn
            | OpCode::BneUnS
    )
}

pub(super) fn is_unconditional_branch(opcode: OpCode) -> bool {
    matches!(opcode, OpCode::Br | OpCode::BrS)
}

pub(super) fn is_leave(opcode: OpCode) -> bool {
    matches!(opcode, OpCode::Leave | OpCode::LeaveS)
}

pub(super) fn get_branch_target(instr: &Instruction) -> Option<i64> {
    match &instr.operand {
        Operand::BranchTarget(target) => Some(*target),
        _ => None,
    }
}

/// Check if an opcode typically produces a statement (empties the stack).
pub(super) fn is_statement_producing(opcode: OpCode) -> bool {
    matches!(
        opcode,
        OpCode::Stloc0
            | OpCode::Stloc1
            | OpCode::Stloc2
            | OpCode::Stloc3
            | OpCode::StlocS
            | OpCode::Stloc
            | OpCode::StargS
            | OpCode::Starg
            | OpCode::Stfld
            | OpCode::Stsfld
            | OpCode::StelemI
            | OpCode::StelemI1
            | OpCode::StelemI2
            | OpCode::StelemI4
            | OpCode::StelemI8
            | OpCode::StelemR4
            | OpCode::StelemR8
            | OpCode::StelemRef
            | OpCode::Stelem
            | OpCode::StindRef
            | OpCode::StindI1
            | OpCode::StindI2
            | OpCode::StindI4
            | OpCode::StindI8
            | OpCode::StindR4
            | OpCode::StindR8
            | OpCode::StindI
            | OpCode::Stobj
            | OpCode::Ret
            | OpCode::Throw
            | OpCode::Rethrow
            | OpCode::Br
            | OpCode::BrS
            | OpCode::Brfalse
            | OpCode::BrfalseS
            | OpCode::Brtrue
            | OpCode::BrtrueS
            | OpCode::Beq
            | OpCode::BeqS
            | OpCode::Bge
            | OpCode::BgeS
            | OpCode::Bgt
            | OpCode::BgtS
            | OpCode::Ble
            | OpCode::BleS
            | OpCode::Blt
            | OpCode::BltS
            | OpCode::BneUn
            | OpCode::BneUnS
            | OpCode::BgeUn
            | OpCode::BgeUnS
            | OpCode::BgtUn
            | OpCode::BgtUnS
            | OpCode::BleUn
            | OpCode::BleUnS
            | OpCode::BltUn
            | OpCode::BltUnS
            | OpCode::Leave
            | OpCode::LeaveS
            | OpCode::Endfinally
            | OpCode::Endfilter
            | OpCode::Switch
            | OpCode::Pop
            | OpCode::Nop
    )
}

pub(super) fn negate_condition(cond: Expr) -> Expr {
    match cond {
        Expr::Unary(UnaryOp::LogicalNot, inner) => *inner,
        Expr::Binary(l, BinOp::Eq, r) => Expr::Binary(l, BinOp::Ne, r),
        Expr::Binary(l, BinOp::Ne, r) => Expr::Binary(l, BinOp::Eq, r),
        Expr::Binary(l, BinOp::Lt, r) => Expr::Binary(l, BinOp::Ge, r),
        Expr::Binary(l, BinOp::Ge, r) => Expr::Binary(l, BinOp::Lt, r),
        Expr::Binary(l, BinOp::Gt, r) => Expr::Binary(l, BinOp::Le, r),
        Expr::Binary(l, BinOp::Le, r) => Expr::Binary(l, BinOp::Gt, r),
        other => Expr::Unary(UnaryOp::LogicalNot, Box::new(other)),
    }
}
