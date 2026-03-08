//! Core simulation helpers: stack primitives, argument/local accessors, and
//! the top-level `process` dispatcher that routes each IL opcode to a handler.

use super::*;

impl<'a> StackSimulator<'a> {
    pub(super) fn pop(&mut self) -> Expr {
        self.stack
            .pop()
            .unwrap_or(Expr::Raw("/* empty stack */".into()))
    }

    pub(super) fn push(&mut self, expr: Expr) {
        self.stack.push(expr);
    }

    pub(super) fn emit(&mut self, stmt: Statement) {
        self.statements.push(stmt);
    }

    pub(super) fn load_arg(&self, index: u16) -> Expr {
        if !self.is_static && index == 0 {
            Expr::This
        } else {
            let adjusted = if self.is_static { index } else { index - 1 };
            let name = self
                .params
                .get(adjusted as usize)
                .cloned()
                .unwrap_or_else(|| format!("arg{}", index));
            Expr::Arg(index, name)
        }
    }

    pub(super) fn load_local(&self, index: u16) -> Expr {
        let name = self
            .local_names
            .get(index as usize)
            .cloned()
            .unwrap_or_else(|| format!("V_{}", index));
        Expr::Local(index, name)
    }

    pub(super) fn store_local(&mut self, index: u16) {
        let value = self.pop();
        let target = self.load_local(index);
        self.emit(Statement::Assign(target, value));
    }

    pub(super) fn store_arg(&mut self, index: u16) {
        let value = self.pop();
        let target = self.load_arg(index);
        self.emit(Statement::Assign(target, value));
    }

    pub(super) fn binary_op(&mut self, op: BinOp) {
        let rhs = self.pop();
        let lhs = self.pop();
        self.push(Expr::Binary(Box::new(lhs), op, Box::new(rhs)));
    }

    /// Process a single IL instruction — dispatches to sub-handlers by opcode category.
    pub(super) fn process(&mut self, instr: &Instruction) {
        if self.process_loads_and_stores(instr) {
            return;
        }
        if self.process_fields_and_calls(instr) {
            return;
        }
        if self.process_arith_and_types(instr) {
            return;
        }
        self.process_arrays_and_control(instr);
    }
}
