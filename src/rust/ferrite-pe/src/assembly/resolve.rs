use dotnetdll::prelude::*;

use super::formatting::format_types::{format_method_ref_parent, format_method_type};

/// Resolve a MethodSource to a qualified name like "TypeName::MethodName".
pub(super) fn resolve_method_source_name(ms: &MethodSource, res: &Resolution) -> String {
    match ms {
        MethodSource::User(um) => resolve_user_method_name(um, res),
        MethodSource::Generic(gi) => {
            let base_name = resolve_user_method_name(&gi.base, res);
            if gi.parameters.is_empty() {
                base_name
            } else {
                let clean = if let Some(idx) = base_name.find('`') {
                    &base_name[..idx]
                } else {
                    &base_name
                };
                let params: Vec<String> = gi
                    .parameters
                    .iter()
                    .map(|p| format_method_type(p, res))
                    .collect();
                format!("{}<{}>", clean, params.join(", "))
            }
        }
    }
}

/// Resolve a UserMethod to "TypeName::MethodName".
pub(super) fn resolve_user_method_name(um: &UserMethod, res: &Resolution) -> String {
    match um {
        UserMethod::Definition(idx) => {
            let method = &res[*idx];
            let parent_td = &res[idx.parent_type()];
            format!("{}::{}", parent_td.name, method.name)
        }
        UserMethod::Reference(idx) => {
            let methref = &res[*idx];
            let parent_name = format_method_ref_parent(&methref.parent, res);
            if parent_name.is_empty() {
                methref.name.to_string()
            } else {
                format!("{}::{}", parent_name, methref.name)
            }
        }
    }
}

/// Resolve a FieldSource to "TypeName::FieldName".
pub(super) fn resolve_field_source_name(fs: &FieldSource, res: &Resolution) -> String {
    match fs {
        FieldSource::Definition(idx) => {
            let field = &res[*idx];
            let parent_td = &res[idx.parent_type()];
            format!("{}::{}", parent_td.name, field.name)
        }
        FieldSource::Reference(idx) => {
            let fieldref = &res[*idx];
            let parent_name = match &fieldref.parent {
                dotnetdll::resolved::members::FieldReferenceParent::Type(mt) => {
                    format_method_type(mt, res)
                }
                dotnetdll::resolved::members::FieldReferenceParent::Module(_) => String::new(),
            };
            if parent_name.is_empty() {
                fieldref.name.to_string()
            } else {
                format!("{}::{}", parent_name, fieldref.name)
            }
        }
    }
}

/// Resolve method call info from a MethodSource: (name, param_count, has_return, is_static).
pub(super) fn resolve_method_source_call_info(
    ms: &MethodSource,
    res: &Resolution,
) -> (String, usize, bool, bool) {
    let name = resolve_method_source_name(ms, res);
    match ms {
        MethodSource::User(um) => {
            let (param_count, has_return, is_static) = match um {
                UserMethod::Definition(idx) => {
                    let method = &res[*idx];
                    (
                        method.signature.parameters.len(),
                        method.signature.return_type.1.is_some(),
                        method.is_static(),
                    )
                }
                UserMethod::Reference(idx) => {
                    let methref = &res[*idx];
                    (
                        methref.signature.parameters.len(),
                        methref.signature.return_type.1.is_some(),
                        !methref.signature.instance,
                    )
                }
            };
            (name, param_count, has_return, is_static)
        }
        MethodSource::Generic(gi) => {
            let (_, param_count, has_return, is_static) =
                resolve_method_source_call_info(&MethodSource::User(gi.base), res);
            (name, param_count, has_return, is_static)
        }
    }
}
