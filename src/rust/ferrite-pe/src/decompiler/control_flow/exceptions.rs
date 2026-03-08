//! Switch and exception handler reconstruction.
//!
//! Contains `try_switch_with_val` and `try_exception_handler`, which convert
//! switch tables and try/catch/finally IL regions into structured AST nodes.

use super::*;

impl<'a> ControlFlowAnalyzer<'a> {
    pub(super) fn try_switch_with_val(
        &self,
        switch_idx: usize,
        region_end: usize,
        depth: usize,
        switch_expr: Expr,
    ) -> Option<StructuredResult> {
        let instr = &self.instructions[switch_idx];
        let targets = match &instr.operand {
            Operand::Switch(targets) => targets.clone(),
            _ => return None,
        };

        if targets.is_empty() {
            return None;
        }

        let mut target_indices: Vec<usize> = Vec::new();
        for &t in &targets {
            let t_offset = t as u32;
            if let Some(&idx) = self.offset_to_index.get(&t_offset) {
                target_indices.push(idx);
            } else {
                return None;
            }
        }

        // Check for default branch (br after switch).
        let after_switch = switch_idx + 1;
        let default_target_idx = if after_switch < region_end {
            let next_opcode = self.instructions[after_switch].opcode;
            if is_unconditional_branch(next_opcode) {
                get_branch_target(&self.instructions[after_switch])
                    .and_then(|t| self.offset_to_index.get(&(t as u32)).copied())
            } else {
                None
            }
        } else {
            None
        };

        let all_targets: Vec<usize> = {
            let mut all = target_indices.clone();
            if let Some(dt) = default_target_idx {
                all.push(dt);
            }
            all.sort();
            all.dedup();
            all
        };

        let switch_end = all_targets
            .iter()
            .copied()
            .max()
            .unwrap_or(region_end)
            .min(region_end);

        let mut cases: Vec<(Vec<Expr>, Block)> = Vec::new();
        let mut default_block: Option<Block> = None;

        let mut sorted_targets: Vec<(usize, usize)> = target_indices
            .iter()
            .enumerate()
            .map(|(case_val, &target)| (case_val, target))
            .collect();
        sorted_targets.sort_by_key(|&(_, target)| target);

        let mut j = 0;
        while j < sorted_targets.len() {
            let target = sorted_targets[j].1;
            let mut labels = vec![Expr::Int(sorted_targets[j].0 as i64)];

            while j + 1 < sorted_targets.len() && sorted_targets[j + 1].1 == target {
                j += 1;
                labels.push(Expr::Int(sorted_targets[j].0 as i64));
            }

            let body_end = if j + 1 < sorted_targets.len() {
                sorted_targets[j + 1].1
            } else {
                switch_end
            };

            let body = self.process_region_impl(target, body_end, depth + 1);
            cases.push((labels, body));
            j += 1;
        }

        if let Some(dt) = default_target_idx {
            let dt_end = switch_end;
            if dt < dt_end {
                default_block = Some(self.process_region_impl(dt, dt_end, depth + 1));
            }
        }

        Some(StructuredResult {
            stmt: Statement::Switch(switch_expr, cases, default_block),
            next_idx: switch_end,
        })
    }

    pub(super) fn try_exception_handler(
        &self,
        idx: usize,
        region_end: usize,
        offset: u32,
        depth: usize,
    ) -> Option<StructuredResult> {
        let consumed = self.consumed_handlers.borrow();
        let handlers: Vec<(usize, &ExceptionHandler)> = self
            .exception_handlers
            .iter()
            .enumerate()
            .filter(|(i, eh)| eh.try_offset == offset && !consumed.contains(i))
            .collect();
        drop(consumed);

        if handlers.is_empty() {
            return None;
        }

        {
            let mut consumed = self.consumed_handlers.borrow_mut();
            for &(i, _) in &handlers {
                consumed.insert(i);
            }
        }

        let try_offset = handlers[0].1.try_offset;
        let try_length = handlers[0].1.try_length;
        let try_end_offset = try_offset + try_length;

        let try_start_idx = idx;
        let try_end_idx = self
            .offset_to_index
            .get(&try_end_offset)
            .copied()
            .unwrap_or(region_end);

        let try_body = self.process_region_impl(try_start_idx, try_end_idx, depth + 1);

        let mut catches = Vec::new();
        let mut finally_block = None;
        let mut max_handler_end = try_end_idx;

        for &(_, eh) in &handlers {
            let handler_start_offset = eh.handler_offset;
            let handler_end_offset = eh.handler_offset + eh.handler_length;

            let handler_start = self
                .offset_to_index
                .get(&handler_start_offset)
                .copied()
                .unwrap_or(try_end_idx);
            let handler_end = self
                .offset_to_index
                .get(&handler_end_offset)
                .copied()
                .unwrap_or(region_end);

            if handler_end > max_handler_end {
                max_handler_end = handler_end;
            }

            match eh.kind {
                ExceptionHandlerKind::Catch => {
                    let exception_type = if eh.class_token_or_filter != 0 {
                        self.resolver.resolve_token(eh.class_token_or_filter)
                    } else {
                        "Exception".to_string()
                    };
                    // The CLR pushes the caught exception onto the stack at catch entry.
                    // Pre-seed the sub-region with an exception expression so the first
                    // instruction (usually stloc or pop) can consume it.
                    let ex_var = "ex".to_string();
                    let init_stack = vec![Expr::Raw(ex_var.clone())];
                    let body = self.process_region_with_stack(
                        handler_start,
                        handler_end,
                        depth + 1,
                        &init_stack,
                    );
                    catches.push(CatchClause {
                        exception_type,
                        var_name: Some(ex_var),
                        body,
                    });
                }
                ExceptionHandlerKind::Finally => {
                    let body = self.process_region_impl(handler_start, handler_end, depth + 1);
                    finally_block = Some(body);
                }
                ExceptionHandlerKind::Fault => {
                    let body = self.process_region_impl(handler_start, handler_end, depth + 1);
                    finally_block = Some(body);
                }
                ExceptionHandlerKind::Filter => {
                    let body = self.process_region_impl(handler_start, handler_end, depth + 1);
                    catches.push(CatchClause {
                        exception_type: "Exception /* filter */".to_string(),
                        var_name: None,
                        body,
                    });
                }
            }
        }

        Some(StructuredResult {
            stmt: Statement::Try(try_body, catches, finally_block),
            next_idx: max_handler_end,
        })
    }
}
