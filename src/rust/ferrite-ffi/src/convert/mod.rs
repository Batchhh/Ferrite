use std::collections::BTreeMap;

use ferrite_pe::assembly::{Assembly, TypeKind as PeTypeKind};

use crate::types::*;

mod flags;
pub(crate) use flags::*;

mod summary;
pub(crate) use summary::*;

/// Convert a parsed assembly to its FFI form, grouping types by namespace.
pub(crate) fn convert_assembly(asm: &Assembly) -> AssemblyInfo {
    let mut ns_map: BTreeMap<String, Vec<TypeInfo>> = BTreeMap::new();

    for td in &asm.types {
        let ns = if td.namespace.is_empty() {
            "<global>".to_string()
        } else {
            td.namespace.to_string()
        };
        let type_info = convert_type(td);
        ns_map.entry(ns).or_default().push(type_info);
    }

    let namespaces = ns_map
        .into_iter()
        .map(|(name, types)| NamespaceInfo { name, types })
        .collect();

    let assembly_references = asm
        .assembly_refs
        .iter()
        .map(|r| format!("{} {}", r.name, r.version))
        .collect();

    AssemblyInfo {
        name: asm.name.clone(),
        version: asm.version.clone(),
        target_framework: asm.target_framework.clone(),
        namespaces,
        assembly_references,
    }
}

/// Convert a single [`TypeDef`] to its FFI representation, including members and nested types.
pub(crate) fn convert_type(td: &ferrite_pe::TypeDef) -> TypeInfo {
    let full_name = if td.namespace.is_empty() {
        td.name.to_string()
    } else {
        format!("{}.{}", td.namespace, td.name)
    };

    let kind = match td.kind {
        PeTypeKind::Class => TypeKind::Class,
        PeTypeKind::Interface => TypeKind::Interface,
        PeTypeKind::Struct => TypeKind::Struct,
        PeTypeKind::Enum => TypeKind::Enum,
        PeTypeKind::Delegate => TypeKind::Delegate,
    };

    let attributes = decode_type_attributes(td.flags);

    let mut members = Vec::with_capacity(td.fields.len() + td.methods.len());

    for field in &td.fields {
        let field_attrs: Vec<AttributeInfo> = field
            .custom_attributes
            .iter()
            .map(|a| AttributeInfo {
                name: a.name.clone(),
                arguments: a.arguments.clone(),
            })
            .collect();

        members.push(MemberInfo {
            name: field.name.to_string(),
            kind: MemberKind::Field,
            token: field.token,
            signature: String::new(),
            method_attributes: None,
            field_attributes: Some(decode_field_attributes(field.flags)),
            return_type: String::new(),
            parameters: Vec::new(),
            attributes_list: field_attrs,
            field_type: field.field_type.clone(),
            constant_value: field.constant_value.clone(),
        });
    }

    for method in &td.methods {
        let params: Vec<ParameterInfo> = method
            .params
            .iter()
            .map(|p| ParameterInfo {
                name: p.name.clone(),
                type_name: p.type_name.clone(),
            })
            .collect();

        let method_attrs: Vec<AttributeInfo> = method
            .custom_attributes
            .iter()
            .map(|a| AttributeInfo {
                name: a.name.clone(),
                arguments: a.arguments.clone(),
            })
            .collect();

        members.push(MemberInfo {
            name: method.name.to_string(),
            kind: MemberKind::Method,
            token: method.token,
            signature: String::new(),
            method_attributes: Some(decode_method_attributes(method.flags, &method.name)),
            field_attributes: None,
            return_type: method.return_type.clone(),
            parameters: params,
            attributes_list: method_attrs,
            field_type: String::new(),
            constant_value: None,
        });
    }

    let properties: Vec<PropertyInfo> = td
        .properties
        .iter()
        .map(|p| {
            let prop_attrs: Vec<AttributeInfo> = p
                .custom_attributes
                .iter()
                .map(|a| AttributeInfo {
                    name: a.name.clone(),
                    arguments: a.arguments.clone(),
                })
                .collect();
            PropertyInfo {
                name: p.name.to_string(),
                token: p.token,
                property_type: p.property_type.clone(),
                getter_token: p.getter_token,
                setter_token: p.setter_token,
                attributes_list: prop_attrs,
            }
        })
        .collect();

    let events: Vec<EventInfo> = td
        .events
        .iter()
        .map(|e| {
            let event_attrs: Vec<AttributeInfo> = e
                .custom_attributes
                .iter()
                .map(|a| AttributeInfo {
                    name: a.name.clone(),
                    arguments: a.arguments.clone(),
                })
                .collect();
            EventInfo {
                name: e.name.to_string(),
                token: e.token,
                event_type: e.event_type.clone(),
                add_token: e.add_token,
                remove_token: e.remove_token,
                raise_token: e.raise_token,
                attributes_list: event_attrs,
            }
        })
        .collect();

    let nested_types = td.nested_types.iter().map(convert_type).collect();

    let type_attrs: Vec<AttributeInfo> = td
        .custom_attributes
        .iter()
        .map(|a| AttributeInfo {
            name: a.name.clone(),
            arguments: a.arguments.clone(),
        })
        .collect();

    TypeInfo {
        name: td.name.to_string(),
        full_name,
        kind,
        token: td.token,
        namespace: td.namespace.to_string(),
        attributes,
        members,
        properties,
        events,
        nested_types,
        base_type: td.base_type.clone(),
        interfaces: td.interfaces.clone(),
        attributes_list: type_attrs,
    }
}

/// Find a type by token (searching nested types) and convert to FFI form.
pub(crate) fn find_type_and_convert(asm: &Assembly, token: u32) -> Option<TypeInfo> {
    fn find_in(types: &[ferrite_pe::TypeDef], token: u32) -> Option<&ferrite_pe::TypeDef> {
        for td in types {
            if td.token == token {
                return Some(td);
            }
            if let Some(found) = find_in(&td.nested_types, token) {
                return Some(found);
            }
        }
        None
    }
    find_in(&asm.types, token).map(convert_type)
}
