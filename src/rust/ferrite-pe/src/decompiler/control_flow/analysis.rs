//! Core region processor and simulator helpers for control flow analysis.
//!
//! This sub-module owns the main `process_region_impl` / `process_region_with_stack`
//! driver loop plus all supporting helpers: linear-end detection, simulator
//! creation utilities, and branch-condition extraction.

use super::*;

impl<'a> ControlFlowAnalyzer<'a> {
    pub(super) fn process_region_impl(
        &self,
        start_idx: usize,
        end_idx: usize,
        depth: usize,
    ) -> Vec<Statement> {
        self.process_region_with_stack(start_idx, end_idx, depth, &[])
    }

    pub(super) fn process_region_with_stack(
        &self,
        start_idx: usize,
        end_idx: usize,
        depth: usize,
        init_stack: &[Expr],
    ) -> Vec<Statement> {
        if depth >= Self::MAX_DEPTH || start_idx >= end_idx {
            return self.simulate_range_fresh(start_idx, end_idx);
        }

        // Create a shared simulator that persists across linear blocks within this region.
        let mut sim = StackSimulator::new(
            self.resolver,
            self.locals.clone(),
            self.params.clone(),
            self.is_static,
        );
        for expr in init_stack {
            sim.push_expr(expr.clone());
        }
        let mut stmts = Vec::new();
        let mut i = start_idx;

        while i < end_idx {
            // 1. Check if an exception handler's try block starts here.
            let offset = self.instructions[i].offset;
            if let Some(try_result) = self.try_exception_handler(i, end_idx, offset, depth) {
                stmts.push(try_result.stmt);
                i = try_result.next_idx.max(i + 1);
                continue;
            }

            // 2. Check if current instruction is a loop header.
            if let Some(back_edge_idx) = self.find_back_edge_to(i, end_idx) {
                if let Some(loop_result) = self.try_loop_at_header(i, back_edge_idx, end_idx, depth)
                {
                    stmts.extend(loop_result.stmts);
                    i = loop_result.next_idx.max(i + 1);
                    continue;
                }
            }

            let opcode = self.instructions[i].opcode;

            // 3. Conditional branch → extract condition from shared sim, try if/else.
            if is_conditional_branch(opcode) {
                let condition = self.extract_branch_condition(&mut sim, i);
                if let Some(flow) =
                    self.try_if_else_with_condition(i, end_idx, depth, condition, &mut sim)
                {
                    match flow {
                        FlowResult::Statement(stmt, next_idx) => {
                            stmts.push(stmt);
                            i = next_idx.max(i + 1);
                        }
                        FlowResult::TernaryPushed(next_idx) => {
                            // Ternary value was pushed onto the shared sim's stack.
                            i = next_idx.max(i + 1);
                        }
                    }
                    continue;
                }
                // If/else detection failed — condition already consumed, skip branch.
                i += 1;
                continue;
            }

            // 4. Switch instruction.
            if opcode == OpCode::Switch {
                let switch_val = sim.pop_condition();
                if let Some(switch_result) = self.try_switch_with_val(i, end_idx, depth, switch_val)
                {
                    stmts.push(switch_result.stmt);
                    i = switch_result.next_idx.max(i + 1);
                    continue;
                }
                // Failed — skip.
                i += 1;
                continue;
            }

            // 5. Unconditional backward branch (not caught as loop) — skip.
            if is_unconditional_branch(opcode) {
                if let Some(target_offset) = get_branch_target(&self.instructions[i]) {
                    if target_offset >= 0 && (target_offset as u32) < self.instructions[i].offset {
                        i += 1;
                        continue;
                    }
                }
            }

            // 6. Linear fallback: process with the shared simulator.
            let linear_end = self.find_linear_end(i, end_idx);
            let linear_end = linear_end.max(i + 1);
            let linear_stmts = sim.simulate_range(self.instructions, i, linear_end);
            stmts.extend(linear_stmts);
            i = linear_end;
        }

        // Drain remaining stack values as expression statements.
        let remaining = sim.drain_stack();
        for expr in remaining {
            stmts.push(Statement::Expr(expr));
        }

        stmts
    }

    /// Find the end of a linear (non-branching) sequence starting at `start`.
    pub(super) fn find_linear_end(&self, start: usize, end_idx: usize) -> usize {
        let mut i = start;
        while i < end_idx {
            let opcode = self.instructions[i].opcode;
            if i > start
                && (is_conditional_branch(opcode)
                    || is_unconditional_branch(opcode)
                    || is_leave(opcode)
                    || opcode == OpCode::Switch)
            {
                return i;
            }
            if i > start {
                let offset = self.instructions[i].offset;
                if self.exception_handler_starts_at(offset) {
                    return i;
                }
            }
            i += 1;
        }
        i
    }

    /// Check if any exception handler's try region starts at the given byte offset.
    pub(super) fn exception_handler_starts_at(&self, offset: u32) -> bool {
        self.exception_handlers
            .iter()
            .any(|eh| eh.try_offset == offset)
    }

    /// Create a fresh simulator and process [start, end) — used for sub-regions
    /// that need their own stack (loop bodies, try bodies, etc.).
    pub(super) fn simulate_range_fresh(&self, start: usize, end: usize) -> Vec<Statement> {
        if start >= end {
            return Vec::new();
        }
        let mut sim = StackSimulator::new(
            self.resolver,
            self.locals.clone(),
            self.params.clone(),
            self.is_static,
        );
        sim.simulate_range(self.instructions, start, end)
    }

    /// Simulate a range with a fresh simulator pre-seeded with initial stack values.
    pub(super) fn simulate_range_with_init_stack(
        &self,
        start: usize,
        end: usize,
        init_stack: &[Expr],
    ) -> (Vec<Statement>, Vec<Expr>) {
        if start >= end {
            return (Vec::new(), init_stack.to_vec());
        }
        let mut sim = StackSimulator::new(
            self.resolver,
            self.locals.clone(),
            self.params.clone(),
            self.is_static,
        );
        for expr in init_stack {
            sim.push_expr(expr.clone());
        }
        let stmts = sim.simulate_range(self.instructions, start, end);
        let stack = sim.drain_stack();
        (stmts, stack)
    }

    /// Extract the condition expression from the shared simulator for a branch instruction.
    /// For brfalse/brtrue: pops 1 value. For binary branches (beq, bgt, etc.): pops 2 values.
    /// Returns the condition under which the fall-through path executes.
    pub(super) fn extract_branch_condition(
        &self,
        sim: &mut StackSimulator,
        branch_idx: usize,
    ) -> Expr {
        let opcode = self.instructions[branch_idx].opcode;
        match opcode {
            OpCode::Brfalse | OpCode::BrfalseS => {
                // brfalse: branch taken when false; fall-through when true
                sim.pop_condition()
            }
            OpCode::Brtrue | OpCode::BrtrueS => {
                // brtrue: branch taken when true; fall-through when false
                negate_condition(sim.pop_condition())
            }
            OpCode::Beq | OpCode::BeqS => {
                let v2 = sim.pop_condition();
                let v1 = sim.pop_condition();
                Expr::Binary(Box::new(v1), BinOp::Eq, Box::new(v2))
            }
            OpCode::Bge | OpCode::BgeS | OpCode::BgeUn | OpCode::BgeUnS => {
                let v2 = sim.pop_condition();
                let v1 = sim.pop_condition();
                Expr::Binary(Box::new(v1), BinOp::Ge, Box::new(v2))
            }
            OpCode::Bgt | OpCode::BgtS | OpCode::BgtUn | OpCode::BgtUnS => {
                let v2 = sim.pop_condition();
                let v1 = sim.pop_condition();
                Expr::Binary(Box::new(v1), BinOp::Gt, Box::new(v2))
            }
            OpCode::Ble | OpCode::BleS | OpCode::BleUn | OpCode::BleUnS => {
                let v2 = sim.pop_condition();
                let v1 = sim.pop_condition();
                Expr::Binary(Box::new(v1), BinOp::Le, Box::new(v2))
            }
            OpCode::Blt | OpCode::BltS | OpCode::BltUn | OpCode::BltUnS => {
                let v2 = sim.pop_condition();
                let v1 = sim.pop_condition();
                Expr::Binary(Box::new(v1), BinOp::Lt, Box::new(v2))
            }
            OpCode::BneUn | OpCode::BneUnS => {
                let v2 = sim.pop_condition();
                let v1 = sim.pop_condition();
                Expr::Binary(Box::new(v1), BinOp::Ne, Box::new(v2))
            }
            _ => Expr::Raw("/* unknown branch condition */".into()),
        }
    }

    /// Build a branch condition using a fresh simulator (for loops where the
    /// condition is inside the loop body, not the parent's shared sim).
    pub(super) fn build_branch_condition_fresh(&self, branch_idx: usize) -> Expr {
        let start = self.find_condition_start(branch_idx);
        let mut sim = StackSimulator::new(
            self.resolver,
            self.locals.clone(),
            self.params.clone(),
            self.is_static,
        );
        let _stmts = sim.simulate_range(self.instructions, start, branch_idx);
        self.extract_branch_condition(&mut sim, branch_idx)
    }

    /// Find the start index for condition evaluation (for fresh-sim condition extraction).
    pub(super) fn find_condition_start(&self, branch_idx: usize) -> usize {
        let mut i = branch_idx;
        while i > 0 {
            i -= 1;
            let opcode = self.instructions[i].opcode;
            if is_statement_producing(opcode) {
                return i + 1;
            }
        }
        0
    }
}
