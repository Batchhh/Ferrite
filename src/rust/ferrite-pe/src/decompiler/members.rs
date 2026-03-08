use super::header::emit_custom_attributes;
use super::methods::{decompile_method_body, method_visibility};
pub(super) use super::methods::{emit_constructor, emit_method};
use super::INDENT;
use crate::assembly::{Assembly, FieldDef, PropertyDef, TypeDef};
use crate::decompiler::patterns;

// ---------------------------------------------------------------------------
// Fields
// ---------------------------------------------------------------------------

pub(super) fn emit_field(field: &FieldDef, hoisted_init: Option<&str>, out: &mut String) {
    out.push_str(INDENT);

    let vis = field_visibility(field.flags);
    if !vis.is_empty() {
        out.push_str(vis);
        out.push(' ');
    }

    if (field.flags & 0x0010) != 0 {
        out.push_str("static ");
    }
    if (field.flags & 0x0020) != 0 {
        out.push_str("readonly ");
    }
    // Literal (const)
    if (field.flags & 0x0040) != 0 {
        out.push_str("const ");
    }

    out.push_str(&field.field_type);
    out.push(' ');
    out.push_str(&field.name);

    if let Some(ref val) = field.constant_value {
        out.push_str(" = ");
        out.push_str(val);
    } else if let Some(init) = hoisted_init {
        out.push_str(" = ");
        out.push_str(init);
    }

    out.push_str(";\n");
}

pub(super) fn field_visibility(flags: u16) -> &'static str {
    match flags & 0x0007 {
        0x0001 => "private",
        0x0002 => "private protected",
        0x0003 => "internal",
        0x0004 => "protected",
        0x0005 => "protected internal",
        0x0006 => "public",
        _ => "private",
    }
}

// ---------------------------------------------------------------------------
// Properties
// ---------------------------------------------------------------------------

pub(super) fn emit_property(
    prop: &PropertyDef,
    td: &TypeDef,
    assembly: &Assembly,
    lambda_map: &patterns::LambdaMap,
    out: &mut String,
) {
    // Emit custom attributes on the property
    emit_custom_attributes(&prop.custom_attributes, out, INDENT);

    out.push_str(INDENT);

    // Derive visibility/modifiers from getter (or setter) method
    let accessor_method = prop
        .getter_token
        .and_then(|t| td.methods.iter().find(|m| m.token == t))
        .or_else(|| {
            prop.setter_token
                .and_then(|t| td.methods.iter().find(|m| m.token == t))
        });

    if let Some(method) = accessor_method {
        let vis = method_visibility(method.flags);
        if !vis.is_empty() {
            out.push_str(vis);
            out.push(' ');
        }
        if (method.flags & 0x0010) != 0 {
            out.push_str("static ");
        }
        if (method.flags & 0x0400) != 0 && (method.flags & 0x0040) == 0 {
            // abstract
            out.push_str("abstract ");
        } else if (method.flags & 0x0040) != 0 && (method.flags & 0x0020) == 0 {
            // virtual but not final
            out.push_str("virtual ");
        }
    }

    out.push_str(&prop.property_type);
    out.push(' ');
    out.push_str(&prop.name);

    // Try to decompile getter/setter bodies
    let getter_method = prop
        .getter_token
        .and_then(|t| td.methods.iter().find(|m| m.token == t));
    let setter_method = prop
        .setter_token
        .and_then(|t| td.methods.iter().find(|m| m.token == t));

    let getter_body = getter_method
        .and_then(|m| decompile_method_body(m, assembly, 3, &td.name, lambda_map).ok());
    let setter_body = setter_method
        .and_then(|m| decompile_method_body(m, assembly, 3, &td.name, lambda_map).ok());

    let has_getter = prop.getter_token.is_some();
    let has_setter = prop.setter_token.is_some();

    // No accessor methods at all — use inline shorthand (nothing to expand)
    if !has_getter && !has_setter {
        out.push_str(" { get; set; }\n");
        return;
    }

    // Always emit full property format — Swift side handles collapse/expand
    out.push('\n');
    out.push_str(INDENT);
    out.push_str("{\n");

    let double_indent = &format!("{}{}", INDENT, INDENT);

    if has_getter {
        if let Some(method) = getter_method {
            emit_custom_attributes(&method.custom_attributes, out, double_indent);
        }
        out.push_str(double_indent);
        out.push_str("get\n");
        out.push_str(double_indent);
        out.push_str("{\n");
        if let Some(ref body) = getter_body {
            out.push_str(body);
        } else {
            out.push_str(INDENT);
            out.push_str(double_indent);
            out.push_str("/* could not decompile getter */\n");
        }
        out.push_str(double_indent);
        out.push_str("}\n");
    }

    if has_setter {
        if let Some(method) = setter_method {
            emit_custom_attributes(&method.custom_attributes, out, double_indent);
        }
        out.push_str(double_indent);
        out.push_str("set\n");
        out.push_str(double_indent);
        out.push_str("{\n");
        if let Some(ref body) = setter_body {
            out.push_str(body);
        } else {
            out.push_str(INDENT);
            out.push_str(double_indent);
            out.push_str("/* could not decompile setter */\n");
        }
        out.push_str(double_indent);
        out.push_str("}\n");
    }

    out.push_str(INDENT);
    out.push_str("}\n");
}
