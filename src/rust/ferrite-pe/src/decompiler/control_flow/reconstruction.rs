//! Structured control flow reconstruction.
//!
//! Converts the raw IL block graph into if/else, while, do-while, switch, and
//! try/catch/finally AST nodes.  All methods are `impl ControlFlowAnalyzer` so
//! they can call the region-processor helpers in `analysis.rs` via `self.`.

use super::*;

impl<'a> ControlFlowAnalyzer<'a> {
    pub(super) fn try_if_else_with_condition(
        &self,
        branch_idx: usize,
        region_end: usize,
        depth: usize,
        condition: Expr,
        sim: &mut StackSimulator,
    ) -> Option<FlowResult> {
        let instr = &self.instructions[branch_idx];
        let target_offset = get_branch_target(instr)? as u32;
        let target_idx = *self.offset_to_index.get(&target_offset)?;

        // Target must be within our region and forward.
        if target_idx > region_end || target_idx <= branch_idx {
            return None;
        }

        let then_start = branch_idx + 1;
        let then_end_raw = target_idx;

        // Check if the then-block ends with an unconditional branch → else block.
        if then_end_raw > then_start {
            let last_then_idx = then_end_raw - 1;
            let last_then_opcode = self.instructions[last_then_idx].opcode;
            if is_unconditional_branch(last_then_opcode) || is_leave(last_then_opcode) {
                if let Some(end_offset) = get_branch_target(&self.instructions[last_then_idx]) {
                    let end_offset = end_offset as u32;
                    if let Some(&end_idx) = self.offset_to_index.get(&end_offset) {
                        if end_idx > target_idx && end_idx <= region_end {
                            // We have an else block:
                            //   then = [then_start..last_then_idx)
                            //   else = [target_idx..end_idx)

                            // --- Ternary detection ---
                            // Drain parent sim's stack to pre-seed branch simulators.
                            // This handles `dup + brtrue` patterns where a duplicated
                            // value on the stack is consumed by the branches.
                            let parent_stack = sim.drain_stack();

                            let (then_stmts, then_stack) = self.simulate_range_with_init_stack(
                                then_start,
                                last_then_idx,
                                &parent_stack,
                            );
                            let (else_stmts, else_stack) = self.simulate_range_with_init_stack(
                                target_idx,
                                end_idx,
                                &parent_stack,
                            );

                            if then_stmts.is_empty()
                                && else_stmts.is_empty()
                                && !then_stack.is_empty()
                                && then_stack.len() == else_stack.len()
                            {
                                // Both branches produced the same number of values and no
                                // statements. The last value on each stack is the ternary
                                // result; the remaining values are unchanged parent values
                                // to restore to the shared sim.
                                let then_val = then_stack.last().unwrap().clone();
                                let else_val = else_stack.last().unwrap().clone();
                                // Restore non-ternary values to the shared sim.
                                for expr in &then_stack[..then_stack.len() - 1] {
                                    sim.push_expr(expr.clone());
                                }
                                let ternary = Expr::Ternary(
                                    Box::new(condition),
                                    Box::new(then_val),
                                    Box::new(else_val),
                                );
                                sim.push_expr(ternary);
                                return Some(FlowResult::TernaryPushed(end_idx));
                            }

                            // Not a ternary — pass parent stack to sub-regions so they
                            // can consume dup'd values (e.g. null-guard patterns).
                            let then_body = self.process_region_with_stack(
                                then_start,
                                last_then_idx,
                                depth + 1,
                                &parent_stack,
                            );
                            let else_body = self.process_region_with_stack(
                                target_idx,
                                end_idx,
                                depth + 1,
                                &parent_stack,
                            );
                            return Some(FlowResult::Statement(
                                Statement::If(condition, then_body, Some(else_body)),
                                end_idx,
                            ));
                        }
                    }
                }
            }
        }

        // No else block — check for short-circuit compound condition pattern.
        //
        // Pattern (short-circuit ||):
        //   [outer] brfalse A → TARGET    (fall-through when A is true)
        //   [inner] ...condition setup...
        //   [guard] brtrue B → EXIT       (skip TARGET when B is true)
        //   TARGET: ...code...
        //   EXIT: ...
        //
        // This is: if (!A || !B) { TARGET code }
        // Combined: !fall_through_cond || inner_fall_through_cond
        if then_end_raw > then_start + 1 {
            let last_idx = then_end_raw - 1;
            if is_conditional_branch(self.instructions[last_idx].opcode) {
                if let Some(guard_target) = get_branch_target(&self.instructions[last_idx]) {
                    if let Some(&guard_exit_idx) = self.offset_to_index.get(&(guard_target as u32))
                    {
                        if guard_exit_idx > then_end_raw && guard_exit_idx <= region_end {
                            // Check that the condition setup has no side effects.
                            let setup_stmts =
                                sim.simulate_range(self.instructions, then_start, last_idx);
                            if setup_stmts.is_empty() {
                                let inner_condition = self.extract_branch_condition(sim, last_idx);

                                // Combined: !outer || inner_fall_through
                                let combined = Expr::Binary(
                                    Box::new(negate_condition(condition)),
                                    BinOp::LogicalOr,
                                    Box::new(inner_condition),
                                );

                                let body = self.process_region_impl(
                                    then_end_raw,
                                    guard_exit_idx,
                                    depth + 1,
                                );
                                return Some(FlowResult::Statement(
                                    Statement::If(combined, body, None),
                                    guard_exit_idx,
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Normal no-else case.
        let then_body = self.process_region_impl(then_start, then_end_raw, depth + 1);
        Some(FlowResult::Statement(
            Statement::If(condition, then_body, None),
            target_idx,
        ))
    }

    /// Find a backward branch in [start+1, end) that targets instruction at `start`.
    pub(super) fn find_back_edge_to(&self, start: usize, end: usize) -> Option<usize> {
        let target_offset = self.instructions[start].offset;
        for j in (start + 1)..end {
            let instr = &self.instructions[j];
            if let Some(branch_target) = get_branch_target(instr) {
                if branch_target >= 0 && branch_target as u32 == target_offset {
                    return Some(j);
                }
            }
        }
        None
    }

    /// Detect a loop starting at `header_idx` with back-edge at `back_edge_idx`.
    /// Loops use fresh simulators for their interior (not the parent's shared sim).
    pub(super) fn try_loop_at_header(
        &self,
        header_idx: usize,
        back_edge_idx: usize,
        region_end: usize,
        depth: usize,
    ) -> Option<LoopResult> {
        let back_opcode = self.instructions[back_edge_idx].opcode;

        // Case 1: Conditional back-edge → do-while loop.
        if is_conditional_branch(back_opcode) {
            let condition = self.build_branch_condition_fresh(back_edge_idx);
            let do_while_cond = negate_condition(condition);
            let body = self.process_region_impl(header_idx, back_edge_idx, depth + 1);
            return Some(LoopResult {
                stmts: vec![Statement::DoWhile(body, do_while_cond)],
                next_idx: back_edge_idx + 1,
            });
        }

        // Case 2: Unconditional back-edge → while or infinite loop.
        if is_unconditional_branch(back_opcode) {
            if let Some(cond_branch_idx) =
                self.find_condition_branch_in_range(header_idx, back_edge_idx)
            {
                let cond_instr = &self.instructions[cond_branch_idx];
                if let Some(exit_offset) = get_branch_target(cond_instr) {
                    let exit_offset_u32 = exit_offset as u32;
                    if exit_offset_u32 > self.instructions[back_edge_idx].offset {
                        let exit_idx = self
                            .offset_to_index
                            .get(&exit_offset_u32)
                            .copied()
                            .unwrap_or(back_edge_idx + 1);

                        let condition = self.build_branch_condition_fresh(cond_branch_idx);
                        let while_cond = condition;

                        let body_start = cond_branch_idx + 1;
                        let body_end = back_edge_idx;
                        let body = self.process_region_impl(body_start, body_end, depth + 1);

                        return Some(LoopResult {
                            stmts: vec![Statement::While(while_cond, body)],
                            next_idx: exit_idx.min(region_end),
                        });
                    }
                }
            }

            // Infinite-style while(true) loop.
            let body = self.process_region_impl(header_idx, back_edge_idx, depth + 1);
            return Some(LoopResult {
                stmts: vec![Statement::While(Expr::Bool(true), body)],
                next_idx: back_edge_idx + 1,
            });
        }

        None
    }

    /// Find the first conditional branch in [start, end).
    pub(super) fn find_condition_branch_in_range(&self, start: usize, end: usize) -> Option<usize> {
        (start..end).find(|&i| is_conditional_branch(self.instructions[i].opcode))
    }
}
