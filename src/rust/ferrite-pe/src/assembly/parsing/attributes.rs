use dotnetdll::prelude::*;

use super::super::formatting::format::format_type_full_name;
use super::super::formatting::format_types::format_method_ref_parent;
use super::super::CustomAttribute;

/// Convert dotnetdll attributes to [`CustomAttribute`] records.
///
/// Skips attributes whose constructor cannot be resolved.
pub(in crate::assembly) fn convert_attributes(
    attrs: &[dotnetdll::resolved::attribute::Attribute],
    res: &Resolution,
) -> Vec<CustomAttribute> {
    use dotnetdll::resolved::types::AlwaysFailsResolver;

    let mut result = Vec::with_capacity(attrs.len());
    for attr in attrs {
        let name = match get_attribute_type_name(attr, res) {
            Some(n) => n,
            None => continue,
        };

        let arguments = match attr.instantiation_data(&AlwaysFailsResolver, res) {
            Ok(data) => {
                let mut args = Vec::new();
                for arg in &data.constructor_args {
                    args.push(format_fixed_arg(arg));
                }
                for arg in &data.named_args {
                    args.push(format_named_arg(arg));
                }
                args
            }
            Err(_) => Vec::new(),
        };

        result.push(CustomAttribute { name, arguments });
    }
    result
}

pub(in crate::assembly) fn convert_field_attributes(
    attrs: &[dotnetdll::resolved::attribute::Attribute],
    res: &Resolution,
) -> Vec<CustomAttribute> {
    convert_attributes(attrs, res)
}

pub(in crate::assembly) fn convert_method_attributes(
    attrs: &[dotnetdll::resolved::attribute::Attribute],
    res: &Resolution,
) -> Vec<CustomAttribute> {
    convert_attributes(attrs, res)
}

pub(in crate::assembly) fn convert_property_attributes(
    attrs: &[dotnetdll::resolved::attribute::Attribute],
    res: &Resolution,
) -> Vec<CustomAttribute> {
    convert_attributes(attrs, res)
}

/// Extract the short display name of an attribute from its constructor reference.
///
/// Strips the namespace prefix and the trailing `Attribute` suffix (e.g. `System.ObsoleteAttribute` → `Obsolete`).
pub(in crate::assembly) fn get_attribute_type_name(
    attr: &dotnetdll::resolved::attribute::Attribute,
    res: &Resolution,
) -> Option<String> {
    let parent_name = match &attr.constructor {
        UserMethod::Reference(idx) => {
            let methref = &res[*idx];
            let full = format_method_ref_parent(&methref.parent, res);
            if full.is_empty() {
                return None;
            }
            full
        }
        UserMethod::Definition(idx) => {
            let parent_td = &res[idx.parent_type()];
            format_type_full_name(parent_td)
        }
    };

    let short = parent_name.rsplit('.').next().unwrap_or(&parent_name);
    let display = short.strip_suffix("Attribute").unwrap_or(short);
    Some(display.to_string())
}

/// Format a fixed (positional) attribute constructor argument as a C# literal.
pub(in crate::assembly) fn format_fixed_arg(
    arg: &dotnetdll::resolved::attribute::FixedArg,
) -> String {
    use dotnetdll::resolved::attribute::FixedArg;
    match arg {
        FixedArg::Boolean(b) => b.to_string(),
        FixedArg::Char(c) => format!("'{}'", c),
        FixedArg::Float32(f) => format!("{}f", f),
        FixedArg::Float64(f) => format!("{}", f),
        FixedArg::String(Some(s)) => format!("\"{}\"", s),
        FixedArg::String(None) => "null".to_string(),
        FixedArg::Integral(i) => format_integral_param(i),
        FixedArg::Enum(name, val) => {
            let short = name.rsplit('.').next().unwrap_or(name);
            format!("{}.{}", short, format_integral_param(val))
        }
        FixedArg::Type(t) => format!("typeof({})", t),
        FixedArg::Array(_, Some(items)) => {
            let strs: Vec<String> = items.iter().map(format_fixed_arg).collect();
            format!("new[] {{ {} }}", strs.join(", "))
        }
        FixedArg::Array(_, None) => "null".to_string(),
        FixedArg::Object(inner) => format_fixed_arg(inner),
    }
}

/// Format a named (field or property) attribute argument as `name = value`.
pub(in crate::assembly) fn format_named_arg(
    arg: &dotnetdll::resolved::attribute::NamedArg,
) -> String {
    use dotnetdll::resolved::attribute::NamedArg;
    match arg {
        NamedArg::Field(name, val) | NamedArg::Property(name, val) => {
            format!("{} = {}", name, format_fixed_arg(val))
        }
    }
}

/// Format an integral attribute argument as its numeric string representation.
pub(in crate::assembly) fn format_integral_param(
    i: &dotnetdll::resolved::attribute::IntegralParam,
) -> String {
    use dotnetdll::resolved::attribute::IntegralParam;
    match i {
        IntegralParam::Int8(v) => v.to_string(),
        IntegralParam::UInt8(v) => v.to_string(),
        IntegralParam::Int16(v) => v.to_string(),
        IntegralParam::UInt16(v) => v.to_string(),
        IntegralParam::Int32(v) => v.to_string(),
        IntegralParam::UInt32(v) => v.to_string(),
        IntegralParam::Int64(v) => v.to_string(),
        IntegralParam::UInt64(v) => v.to_string(),
    }
}

/// Format a field constant value as a C# literal.
pub(in crate::assembly) fn format_constant(c: &Constant) -> String {
    match c {
        Constant::Boolean(b) => b.to_string(),
        Constant::Char(ch) => format!("'{}'", char::from_u32(*ch as u32).unwrap_or('?')),
        Constant::Int8(v) => v.to_string(),
        Constant::UInt8(v) => v.to_string(),
        Constant::Int16(v) => v.to_string(),
        Constant::UInt16(v) => v.to_string(),
        Constant::Int32(v) => v.to_string(),
        Constant::UInt32(v) => v.to_string(),
        Constant::Int64(v) => v.to_string(),
        Constant::UInt64(v) => v.to_string(),
        Constant::Float32(v) => format!("{:.1}f", v),
        Constant::Float64(v) => format!("{:.1}", v),
        Constant::String(chars) => {
            let s = String::from_utf16_lossy(chars);
            format!("\"{}\"", s)
        }
        Constant::Null => "null".into(),
    }
}
