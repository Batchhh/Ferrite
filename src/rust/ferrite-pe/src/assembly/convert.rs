use std::sync::Arc;

use dotnetdll::prelude::*;

use super::attributes::{
    convert_field_attributes, convert_method_attributes, convert_property_attributes,
};
use super::format::simplify_method_name;
use super::format_types::{format_member_type, format_method_type, format_return_type};
use super::method_body::convert_method_body;
use super::{Assembly, FieldDef, MethodDef, MethodDefCallInfo, ParamInfo, PropertyDef};

/// Encode dotnetdll type flags to the raw ECMA-335 TypeDef flags bitmask.
pub(super) fn encode_type_flags(
    flags: &dotnetdll::resolved::types::TypeFlags,
    is_abstract: bool,
    is_sealed: bool,
) -> u32 {
    use dotnetdll::resolved::types::Accessibility as TypeAcc;
    use dotnetdll::resolved::Accessibility as Acc;
    let mut f: u32 = 0;
    f |= match flags.accessibility {
        TypeAcc::NotPublic => 0x00,
        TypeAcc::Public => 0x01,
        TypeAcc::Nested(ref nested) => match nested {
            Acc::Private => 0x03,
            Acc::FamilyANDAssembly => 0x07,
            Acc::Assembly => 0x05,
            Acc::Family => 0x04,
            Acc::FamilyORAssembly => 0x06,
            Acc::Public => 0x02,
        },
    };
    if is_abstract {
        f |= 0x0080;
    }
    if is_sealed {
        f |= 0x0100;
    }
    if matches!(flags.kind, dotnetdll::resolved::types::Kind::Interface) {
        f |= 0x0020;
    }
    f
}

/// Encode dotnetdll method accessibility and modifiers to raw method flags.
pub(super) fn encode_method_flags(method: &dotnetdll::resolved::members::Method) -> u16 {
    use dotnetdll::resolved::members::Accessibility as MemAcc;
    use dotnetdll::resolved::Accessibility as Acc;
    let mut f: u16 = 0;
    f |= match method.accessibility {
        MemAcc::CompilerControlled => 0x0000,
        MemAcc::Access(Acc::Private) => 0x0001,
        MemAcc::Access(Acc::FamilyANDAssembly) => 0x0002,
        MemAcc::Access(Acc::Assembly) => 0x0003,
        MemAcc::Access(Acc::Family) => 0x0004,
        MemAcc::Access(Acc::FamilyORAssembly) => 0x0005,
        MemAcc::Access(Acc::Public) => 0x0006,
    };
    if method.is_static() {
        f |= 0x0010;
    }
    if method.sealed {
        f |= 0x0020;
    }
    if method.virtual_member {
        f |= 0x0040;
    }
    if matches!(method.vtable_layout, VtableLayout::NewSlot) {
        f |= 0x0100;
    }
    if method.abstract_member {
        f |= 0x0400;
    }
    if method.special_name {
        f |= 0x0800;
    }
    if method.pinvoke.is_some() {
        f |= 0x2000;
    }
    f
}

/// Encode dotnetdll method implementation flags (InternalCall, native, runtime body format).
pub(super) fn encode_method_impl_flags(method: &dotnetdll::resolved::members::Method) -> u16 {
    let mut f: u16 = 0;
    if method.internal_call {
        f |= 0x1000;
    }
    match method.body_format {
        BodyFormat::IL => {}
        BodyFormat::Native => {
            f |= 0x0001;
        }
        BodyFormat::Runtime => {
            f |= 0x0003;
        }
    }
    f
}

/// Encode dotnetdll field accessibility and modifiers to raw field flags.
pub(super) fn encode_field_flags(field: &dotnetdll::resolved::members::Field) -> u16 {
    use dotnetdll::resolved::members::Accessibility as MemAcc;
    use dotnetdll::resolved::Accessibility as Acc;
    let mut f: u16 = 0;
    f |= match field.accessibility {
        MemAcc::CompilerControlled => 0x0000,
        MemAcc::Access(Acc::Private) => 0x0001,
        MemAcc::Access(Acc::FamilyANDAssembly) => 0x0002,
        MemAcc::Access(Acc::Assembly) => 0x0003,
        MemAcc::Access(Acc::Family) => 0x0004,
        MemAcc::Access(Acc::FamilyORAssembly) => 0x0005,
        MemAcc::Access(Acc::Public) => 0x0006,
    };
    if field.static_member {
        f |= 0x0010;
    }
    if field.init_only {
        f |= 0x0020;
    }
    if field.literal {
        f |= 0x0040;
    }
    f
}

pub(super) use super::format_types::format_type_source;

/// Extract call-site info (parameter count, has return value, is static) from a method definition.
pub(super) fn method_call_info(method: &dotnetdll::resolved::members::Method) -> MethodDefCallInfo {
    MethodDefCallInfo {
        param_count: method.signature.parameters.len(),
        has_return: method.signature.return_type.1.is_some(),
        is_static: method.is_static(),
    }
}

/// Build a minimal ECMA-335 method signature blob sufficient for call-info extraction.
pub(super) fn encode_method_sig_blob(
    sig: &dotnetdll::resolved::signature::ManagedMethod<MethodType>,
) -> Vec<u8> {
    let mut blob = Vec::new();
    let calling_conv: u8 = if sig.instance { 0x20 } else { 0x00 };
    blob.push(calling_conv);
    blob.push(sig.parameters.len() as u8);
    if sig.return_type.1.is_none() {
        blob.push(0x01); // ELEMENT_TYPE_VOID
    } else {
        blob.push(0x08); // placeholder non-void
    }
    blob
}

/// Convert a dotnetdll field definition to [`FieldDef`].
pub(super) fn convert_field(
    field: &dotnetdll::resolved::members::Field,
    token: u32,
    res: &Resolution,
) -> FieldDef {
    let field_type = format_member_type(&field.return_type, res);
    let constant_value = field
        .default
        .as_ref()
        .map(super::attributes::format_constant);
    let custom_attributes = convert_field_attributes(&field.attributes, res);

    FieldDef {
        token,
        name: Arc::from(field.name.as_ref()),
        flags: encode_field_flags(field),
        field_type,
        constant_value,
        custom_attributes,
    }
}

/// Convert a dotnetdll method definition to [`MethodDef`], parsing the body if present.
pub(super) fn convert_method(
    method: &dotnetdll::resolved::members::Method,
    token: u32,
    res: &Resolution,
    asm: &mut Assembly,
) -> MethodDef {
    let return_type = format_return_type(&method.signature.return_type, res);
    let params: Vec<ParamInfo> = method
        .signature
        .parameters
        .iter()
        .enumerate()
        .map(|(i, param)| {
            let type_name = match &param.1 {
                dotnetdll::resolved::signature::ParameterType::Value(t) => {
                    format_method_type(t, res)
                }
                dotnetdll::resolved::signature::ParameterType::Ref(t) => {
                    format!("ref {}", format_method_type(t, res))
                }
                dotnetdll::resolved::signature::ParameterType::TypedReference => {
                    "TypedReference".into()
                }
            };
            let name = method
                .parameter_metadata
                .get(i)
                .and_then(|m| m.as_ref())
                .and_then(|m| m.name.as_ref())
                .map(|n| n.to_string())
                .unwrap_or_else(|| format!("param{}", i));
            ParamInfo { name, type_name }
        })
        .collect();

    let custom_attributes = convert_method_attributes(&method.attributes, res);

    let has_body = method.body.is_some();
    let parsed_body = method
        .body
        .as_ref()
        .map(|body| convert_method_body(body, res, asm));

    MethodDef {
        token,
        name: Arc::from(simplify_method_name(&method.name)),
        flags: encode_method_flags(method),
        impl_flags: encode_method_impl_flags(method),
        rva: if has_body { 1 } else { 0 }, // Non-zero means has body
        return_type,
        params,
        custom_attributes,
        parsed_body,
    }
}

/// Convert a dotnetdll property definition to [`PropertyDef`].
pub(super) fn convert_property(
    prop: &dotnetdll::resolved::members::Property,
    token: u32,
    getter_token: Option<u32>,
    setter_token: Option<u32>,
    res: &Resolution,
) -> PropertyDef {
    let property_type = match &prop.property_type.1 {
        dotnetdll::resolved::signature::ParameterType::Value(t) => format_member_type(t, res),
        dotnetdll::resolved::signature::ParameterType::Ref(t) => {
            format!("ref {}", format_member_type(t, res))
        }
        dotnetdll::resolved::signature::ParameterType::TypedReference => "TypedReference".into(),
    };

    PropertyDef {
        token,
        name: Arc::from(prop.name.as_ref()),
        property_type,
        getter_token,
        setter_token,
        custom_attributes: convert_property_attributes(&prop.attributes, res),
    }
}

/// Find the token of a property accessor in the already-converted method list.
///
/// Returns `None` if the accessor is not present, indicating it must be converted separately
/// (as happens with Il2CppDumper DLLs that store accessors only on the Property).
pub(super) fn find_accessor_token(
    _source_methods: &[dotnetdll::resolved::members::Method],
    converted_methods: &[MethodDef],
    prop: &dotnetdll::resolved::members::Property,
    is_getter: bool,
) -> Option<u32> {
    let accessor = if is_getter {
        prop.getter.as_ref()?
    } else {
        prop.setter.as_ref()?
    };
    let accessor_name = accessor.name.as_ref();

    converted_methods
        .iter()
        .find(|m| *m.name == *accessor_name)
        .map(|m| m.token)
}
