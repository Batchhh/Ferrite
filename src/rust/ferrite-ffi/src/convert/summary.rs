use std::collections::BTreeMap;

use ferrite_pe::assembly::{Assembly, TypeDef, TypeKind as PeTypeKind};

use super::flags::decode_type_attributes;
use crate::types::*;

fn convert_pe_type_kind(kind: &PeTypeKind) -> TypeKind {
    match kind {
        PeTypeKind::Class => TypeKind::Class,
        PeTypeKind::Interface => TypeKind::Interface,
        PeTypeKind::Struct => TypeKind::Struct,
        PeTypeKind::Enum => TypeKind::Enum,
        PeTypeKind::Delegate => TypeKind::Delegate,
    }
}

/// Convert an assembly to a lightweight summary (no type details).
pub(crate) fn convert_assembly_summary(
    asm: &Assembly,
    id: &str,
    file_path: &str,
) -> AssemblySummary {
    let mut ns_counts: BTreeMap<String, u32> = BTreeMap::new();
    for td in &asm.types {
        let ns = if td.namespace.is_empty() {
            "<global>".to_string()
        } else {
            td.namespace.to_string()
        };
        *ns_counts.entry(ns).or_default() += 1;
    }

    let namespaces = ns_counts
        .into_iter()
        .map(|(name, type_count)| NamespaceSummary { name, type_count })
        .collect();

    let assembly_references = asm
        .assembly_refs
        .iter()
        .map(|r| format!("{} {}", r.name, r.version))
        .collect();

    AssemblySummary {
        id: id.to_string(),
        file_path: file_path.to_string(),
        name: asm.name.clone(),
        version: asm.version.clone(),
        target_framework: asm.target_framework.clone(),
        namespaces,
        assembly_references,
    }
}

/// Convert types in a namespace to lightweight summaries.
pub(crate) fn convert_namespace_types(asm: &Assembly, namespace: &str) -> Vec<TypeSummary> {
    let target_ns = if namespace == "<global>" {
        ""
    } else {
        namespace
    };
    asm.types
        .iter()
        .filter(|td| *td.namespace == *target_ns)
        .map(convert_type_summary)
        .collect()
}

/// Convert a single TypeDef to a lightweight summary.
pub(crate) fn convert_type_summary(td: &TypeDef) -> TypeSummary {
    let full_name = if td.namespace.is_empty() {
        td.name.to_string()
    } else {
        format!("{}.{}", td.namespace, td.name)
    };
    TypeSummary {
        name: td.name.to_string(),
        full_name,
        kind: convert_pe_type_kind(&td.kind),
        token: td.token,
        namespace: td.namespace.to_string(),
        attributes: decode_type_attributes(td.flags),
        member_count: (td.fields.len() + td.methods.len()) as u32,
        property_count: td.properties.len() as u32,
        nested_type_count: td.nested_types.len() as u32,
        base_type: td.base_type.clone(),
        interfaces: td.interfaces.clone(),
    }
}

/// Build flat list of all searchable items (types + members + properties) for the search index.
pub(crate) fn convert_searchable_items(asm: &Assembly) -> Vec<SearchableItem> {
    let mut items = Vec::new();
    collect_searchable_types(&asm.types, &mut items, "");
    items
}

fn collect_searchable_types(types: &[TypeDef], items: &mut Vec<SearchableItem>, parent_ns: &str) {
    for td in types {
        let ns = if td.namespace.is_empty() {
            parent_ns
        } else {
            &td.namespace
        };
        let full_name = if ns.is_empty() {
            td.name.to_string()
        } else {
            format!("{}.{}", ns, td.name)
        };

        let type_kind = match td.kind {
            PeTypeKind::Class => SearchableKind::Class,
            PeTypeKind::Interface => SearchableKind::Interface,
            PeTypeKind::Struct => SearchableKind::Struct,
            PeTypeKind::Enum => SearchableKind::Enum,
            PeTypeKind::Delegate => SearchableKind::Delegate,
        };

        items.push(SearchableItem {
            name: td.name.to_string(),
            full_name,
            kind: type_kind,
            token: td.token,
            parent_token: None,
        });

        // Add members
        for method in &td.methods {
            items.push(SearchableItem {
                name: method.name.to_string(),
                full_name: format!("{}.{}", td.name, method.name),
                kind: SearchableKind::Method,
                token: method.token,
                parent_token: Some(td.token),
            });
        }
        for field in &td.fields {
            let kind = if field.constant_value.is_some() {
                SearchableKind::Constant
            } else {
                SearchableKind::Field
            };
            items.push(SearchableItem {
                name: field.name.to_string(),
                full_name: format!("{}.{}", td.name, field.name),
                kind,
                token: field.token,
                parent_token: Some(td.token),
            });
        }
        for prop in &td.properties {
            items.push(SearchableItem {
                name: prop.name.to_string(),
                full_name: format!("{}.{}", td.name, prop.name),
                kind: SearchableKind::Property,
                token: prop.token,
                parent_token: Some(td.token),
            });
        }

        // Recurse into nested types
        collect_searchable_types(&td.nested_types, items, ns);
    }
}
