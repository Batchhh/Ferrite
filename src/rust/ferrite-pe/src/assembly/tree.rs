use std::collections::HashMap;

use super::TypeDef;

/// Recursively attach nested types and return the top-level type list.
///
/// Consumes the flat vector, using `.take()` to move types out without cloning.
pub(super) fn build_type_tree(
    type_defs_flat: Vec<(usize, TypeDef)>,
    type_encloser_map: &HashMap<usize, usize>,
) -> Vec<TypeDef> {
    let mut nested_map: HashMap<usize, Vec<usize>> = HashMap::new();
    for (&child_idx, &parent_idx) in type_encloser_map {
        nested_map.entry(parent_idx).or_default().push(child_idx);
    }

    let idx_to_pos: HashMap<usize, usize> = type_defs_flat
        .iter()
        .enumerate()
        .map(|(pos, (raw_idx, _))| (*raw_idx, pos))
        .collect();

    // Wrap in Option so we can .take() instead of .clone()
    let mut slots: Vec<(usize, Option<TypeDef>)> = type_defs_flat
        .into_iter()
        .map(|(idx, td)| (idx, Some(td)))
        .collect();

    fn extract_nested(
        raw_idx: usize,
        slots: &mut Vec<(usize, Option<TypeDef>)>,
        nested_map: &HashMap<usize, Vec<usize>>,
        idx_to_pos: &HashMap<usize, usize>,
    ) -> Vec<TypeDef> {
        let children = match nested_map.get(&raw_idx) {
            Some(c) => c.clone(),
            None => return Vec::new(),
        };
        let mut result = Vec::with_capacity(children.len());
        for child_idx in children {
            let grandchildren = extract_nested(child_idx, slots, nested_map, idx_to_pos);
            if let Some(&pos) = idx_to_pos.get(&child_idx) {
                if let Some(mut child_type) = slots[pos].1.take() {
                    child_type.nested_types = grandchildren;
                    result.push(child_type);
                }
            }
        }
        result
    }

    // Process bottom-up: extract nested types first, then collect top-level
    let top_level_indices: Vec<usize> = slots
        .iter()
        .filter(|(raw_idx, _)| !type_encloser_map.contains_key(raw_idx))
        .map(|(raw_idx, _)| *raw_idx)
        .collect();

    let mut result = Vec::with_capacity(top_level_indices.len());
    for raw_idx in top_level_indices {
        let nested = extract_nested(raw_idx, &mut slots, &nested_map, &idx_to_pos);
        if let Some(&pos) = idx_to_pos.get(&raw_idx) {
            if let Some(mut td) = slots[pos].1.take() {
                td.nested_types = nested;
                result.push(td);
            }
        }
    }
    result
}
