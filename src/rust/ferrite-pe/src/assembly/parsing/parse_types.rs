use std::collections::HashMap;
use std::sync::Arc;

use dotnetdll::prelude::*;

use super::super::formatting::{convert, format};
use super::super::{Assembly, StringInterner, TypeDef};
use super::attributes;

/// Parse all type definitions from the resolution into a flat list with their raw indices.
///
/// Also builds the encloser map (child_raw_idx → parent_raw_idx) for nesting.
/// Mutates `asm` to populate method/field/type name tables and call-info tables.
pub(in crate::assembly) fn parse_type_definitions(
    res: &Resolution,
    asm: &mut Assembly,
    interner: &mut StringInterner,
) -> (Vec<(usize, TypeDef)>, HashMap<usize, usize>) {
    let mut global_method_counter: u32 = 1;
    let mut global_field_counter: u32 = 1;
    let mut global_property_counter: u32 = 1;

    // child_raw_idx -> parent_raw_idx, resolved via pointer identity (TypeIndex is opaque)
    let ptr_to_idx: HashMap<*const (), usize> = res
        .type_definitions
        .iter()
        .enumerate()
        .map(|(i, td)| (td as *const _ as *const (), i))
        .collect();

    let mut type_encloser_map: HashMap<usize, usize> = HashMap::new();
    for (child_idx, typedef) in res.type_definitions.iter().enumerate() {
        if let Some(encloser) = typedef.encloser {
            let encloser_td = &res[encloser];
            if let Some(&parent_idx) = ptr_to_idx.get(&(encloser_td as *const _ as *const ())) {
                type_encloser_map.insert(child_idx, parent_idx);
            }
        }
    }

    let num_types = res.type_definitions.len();
    let mut type_defs_flat: Vec<(usize, TypeDef)> = Vec::with_capacity(num_types);

    for (type_raw_idx, typedef) in res.type_definitions.iter().enumerate() {
        // Skip <Module> pseudo-type; still advance token counters.
        if type_raw_idx == 0 && typedef.name == "<Module>" {
            for method in &typedef.methods {
                asm.method_def_names.push(Arc::from(method.name.as_ref()));
                asm.method_def_call_infos
                    .push(convert::method_call_info(method));
                global_method_counter += 1;
            }
            for field in &typedef.fields {
                asm.field_def_names.push(Arc::from(field.name.as_ref()));
                global_field_counter += 1;
            }
            for _prop in &typedef.properties {
                global_property_counter += 1;
            }
            asm.type_def_names.push(Arc::from(typedef.name.as_ref()));
            continue;
        }

        let type_name = interner.intern(&typedef.name);
        asm.type_def_names.push(Arc::clone(&type_name));

        let token = 0x02000000 | ((type_raw_idx as u32) + 1);
        let flags = convert::encode_type_flags(
            &typedef.flags,
            typedef.flags.abstract_type,
            typedef.flags.sealed,
        );

        let kind = format::determine_type_kind(typedef, res);
        let base_type = typedef
            .extends
            .as_ref()
            .map(|ts| convert::format_type_source(ts, res));
        let interfaces: Vec<String> = typedef
            .implements
            .iter()
            .map(|(_, ts)| convert::format_type_source(ts, res))
            .collect();

        let mut fields = Vec::with_capacity(typedef.fields.len());
        for field in &typedef.fields {
            let field_token = 0x04000000 | global_field_counter;
            let field_def = convert::convert_field(field, field_token, res);
            asm.field_def_names.push(Arc::clone(&field_def.name));
            fields.push(field_def);
            global_field_counter += 1;
        }

        let mut methods = Vec::with_capacity(typedef.methods.len());
        for method in &typedef.methods {
            let method_token = 0x06000000 | global_method_counter;
            asm.method_def_call_infos
                .push(convert::method_call_info(method));

            let method_def = convert::convert_method(method, method_token, res, asm);
            asm.method_def_names.push(Arc::clone(&method_def.name));
            methods.push(method_def);
            global_method_counter += 1;
        }

        let mut properties = Vec::with_capacity(typedef.properties.len());
        for prop in typedef.properties.iter() {
            let prop_token = 0x17000000 | global_property_counter;

            // Il2CppDumper DLLs may store accessors only on the Property, not in the
            // methods list, so fall back to converting from the Property's own accessors.

            let getter_token = prop.getter.as_ref().and_then(|getter| {
                convert::find_accessor_token(&typedef.methods, &methods, prop, true).or_else(|| {
                    let token = 0x06000000 | global_method_counter;
                    global_method_counter += 1;
                    asm.method_def_call_infos
                        .push(convert::method_call_info(getter));
                    let method_def = convert::convert_method(getter, token, res, asm);
                    asm.method_def_names.push(Arc::clone(&method_def.name));
                    methods.push(method_def);
                    Some(token)
                })
            });
            let setter_token = prop.setter.as_ref().and_then(|setter| {
                convert::find_accessor_token(&typedef.methods, &methods, prop, false).or_else(
                    || {
                        let token = 0x06000000 | global_method_counter;
                        global_method_counter += 1;
                        asm.method_def_call_infos
                            .push(convert::method_call_info(setter));
                        let method_def = convert::convert_method(setter, token, res, asm);
                        asm.method_def_names.push(Arc::clone(&method_def.name));
                        methods.push(method_def);
                        Some(token)
                    },
                )
            });

            properties.push(convert::convert_property(
                prop,
                prop_token,
                getter_token,
                setter_token,
                res,
            ));
            global_property_counter += 1;
        }

        let custom_attributes = attributes::convert_attributes(&typedef.attributes, res);

        let namespace = typedef
            .namespace
            .as_ref()
            .map(|n| interner.intern(n))
            .unwrap_or_else(|| interner.intern(""));

        type_defs_flat.push((
            type_raw_idx,
            TypeDef {
                token,
                name: type_name,
                namespace,
                kind,
                flags,
                methods,
                fields,
                properties,
                nested_types: Vec::new(),
                base_type,
                interfaces,
                custom_attributes,
            },
        ));
    }

    (type_defs_flat, type_encloser_map)
}
