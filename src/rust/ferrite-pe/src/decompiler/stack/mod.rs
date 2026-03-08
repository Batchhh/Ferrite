//! Stack simulation — converts stack-based IL instructions into expression trees.
//!
//! The simulator walks IL instructions sequentially, maintaining a virtual evaluation
//! stack. Each IL instruction either pushes an expression, pops operands and creates
//! a statement, or both.

use crate::decompiler::ast::*;
use crate::decompiler::resolver::MetadataResolver;
use crate::il::{Instruction, OpCode, Operand};
use std::collections::HashMap;

mod expressions;
mod process_arith;
mod process_load;
mod simulation;

use expressions::generate_local_names;

/// Simulates the .NET evaluation stack to produce C# AST statements.
pub struct StackSimulator<'a> {
    pub(super) resolver: &'a MetadataResolver<'a>,
    pub(super) stack: Vec<Expr>,
    pub(super) statements: Vec<Statement>,
    #[allow(dead_code)]
    pub(super) locals: Vec<String>,
    pub(super) local_names: Vec<String>,
    pub(super) params: Vec<String>,
    pub(super) is_static: bool,
}

impl<'a> StackSimulator<'a> {
    /// Create a new stack simulator. `is_static` controls whether `ldarg.0` maps to `this`.
    pub fn new(
        resolver: &'a MetadataResolver<'a>,
        locals: Vec<String>,
        params: Vec<String>,
        is_static: bool,
    ) -> Self {
        let local_names = generate_local_names(&locals, &params);
        Self {
            resolver,
            stack: Vec::new(),
            statements: Vec::new(),
            locals,
            local_names,
            params,
            is_static,
        }
    }

    /// Main entry point: simulate all instructions and return produced statements.
    #[allow(dead_code)]
    pub fn simulate(&mut self, instructions: &[Instruction]) -> Vec<Statement> {
        for instr in instructions {
            self.process(instr);
        }
        // Drain any remaining stack values as expression statements
        let remaining: Vec<Expr> = self.stack.drain(..).collect();
        for expr in remaining {
            self.statements.push(Statement::Expr(expr));
        }
        std::mem::take(&mut self.statements)
    }

    /// Simulate a range of instructions [start, end) and return produced statements.
    /// Does NOT drain remaining stack values (they may be consumed by branches).
    pub fn simulate_range(
        &mut self,
        instructions: &[Instruction],
        start: usize,
        end: usize,
    ) -> Vec<Statement> {
        let end = end.min(instructions.len());
        for instr in &instructions[start..end] {
            self.process(instr);
        }
        std::mem::take(&mut self.statements)
    }

    /// Pop the top expression from the stack (for use by control flow analyzer
    /// to extract branch conditions).
    pub fn pop_condition(&mut self) -> Expr {
        self.pop()
    }

    #[allow(dead_code)]
    pub fn stack_len(&self) -> usize {
        self.stack.len()
    }

    /// Push an expression onto the stack (public, for control flow to inject values).
    pub fn push_expr(&mut self, expr: Expr) {
        self.stack.push(expr);
    }

    /// Drain all remaining stack values.
    pub fn drain_stack(&mut self) -> Vec<Expr> {
        self.stack.drain(..).collect()
    }
}
