/// Type formatting helpers shared across the assembly sub-modules.
use dotnetdll::prelude::*;

pub(in crate::assembly) fn format_type_source(
    ts: &TypeSource<MemberType>,
    res: &Resolution,
) -> String {
    match ts {
        TypeSource::User(ut) => format_user_type(ut, res),
        TypeSource::Generic { base, parameters } => {
            let base_name = format_user_type(base, res);
            // Strip backtick arity suffix
            let clean = if let Some(idx) = base_name.find('`') {
                &base_name[..idx]
            } else {
                &base_name
            };
            let params: Vec<String> = parameters
                .iter()
                .map(|p| format_member_type(p, res))
                .collect();
            format!("{}<{}>", clean, params.join(", "))
        }
    }
}

pub(in crate::assembly) fn format_user_type(ut: &UserType, res: &Resolution) -> String {
    match ut {
        UserType::Definition(idx) => {
            let td = &res[*idx];
            td.name.to_string()
        }
        UserType::Reference(idx) => {
            let tr = &res[*idx];
            tr.name.to_string()
        }
    }
}

pub(in crate::assembly) fn format_member_type(mt: &MemberType, res: &Resolution) -> String {
    match mt {
        MemberType::Base(base) => format_base_type(base, res),
        MemberType::TypeGeneric(idx) => format!("T{}", idx),
    }
}

pub(in crate::assembly) fn format_method_type(mt: &MethodType, res: &Resolution) -> String {
    match mt {
        MethodType::Base(base) => format_base_type_method(base, res),
        MethodType::TypeGeneric(idx) => format!("T{}", idx),
        MethodType::MethodGeneric(idx) => format!("M{}", idx),
    }
}

pub(in crate::assembly) trait FormatType: Sized {
    fn format(val: &Self, res: &Resolution) -> String;
}

impl FormatType for MemberType {
    fn format(val: &MemberType, res: &Resolution) -> String {
        format_member_type(val, res)
    }
}

impl FormatType for MethodType {
    fn format(val: &MethodType, res: &Resolution) -> String {
        format_method_type(val, res)
    }
}

pub(in crate::assembly) fn format_base_type<T: FormatType + Clone>(
    bt: &BaseType<T>,
    res: &Resolution,
) -> String {
    match bt {
        BaseType::Boolean => "bool".into(),
        BaseType::Char => "char".into(),
        BaseType::Int8 => "sbyte".into(),
        BaseType::UInt8 => "byte".into(),
        BaseType::Int16 => "short".into(),
        BaseType::UInt16 => "ushort".into(),
        BaseType::Int32 => "int".into(),
        BaseType::UInt32 => "uint".into(),
        BaseType::Int64 => "long".into(),
        BaseType::UInt64 => "ulong".into(),
        BaseType::Float32 => "float".into(),
        BaseType::Float64 => "double".into(),
        BaseType::IntPtr => "IntPtr".into(),
        BaseType::UIntPtr => "UIntPtr".into(),
        BaseType::Object => "object".into(),
        BaseType::String => "string".into(),
        BaseType::Type { source, .. } => format_type_source_generic(source, res),
        BaseType::Vector(_, inner) => format!("{}[]", T::format(inner, res)),
        BaseType::Array(inner, _shape) => format!("{}[,]", T::format(inner, res)),
        BaseType::ValuePointer(_, Some(inner)) => format!("{}*", T::format(inner, res)),
        BaseType::ValuePointer(_, None) => "void*".into(),
        BaseType::FunctionPointer(_) => "delegate*".into(),
    }
}

pub(in crate::assembly) fn format_base_type_method(
    bt: &BaseType<MethodType>,
    res: &Resolution,
) -> String {
    format_base_type(bt, res)
}

pub(in crate::assembly) fn format_type_source_generic<T: FormatType + Clone>(
    ts: &TypeSource<T>,
    res: &Resolution,
) -> String {
    match ts {
        TypeSource::User(ut) => format_user_type(ut, res),
        TypeSource::Generic { base, parameters } => {
            let base_name = format_user_type(base, res);
            let clean = if let Some(idx) = base_name.find('`') {
                &base_name[..idx]
            } else {
                &base_name
            };
            let params: Vec<String> = parameters.iter().map(|p| T::format(p, res)).collect();
            format!("{}<{}>", clean, params.join(", "))
        }
    }
}

pub(in crate::assembly) fn format_return_type(
    rt: &dotnetdll::resolved::signature::ReturnType<MethodType>,
    res: &Resolution,
) -> String {
    match &rt.1 {
        None => "void".into(),
        Some(pt) => format_parameter_type(pt, res),
    }
}

pub(in crate::assembly) fn format_parameter_type(
    pt: &dotnetdll::resolved::signature::ParameterType<MethodType>,
    res: &Resolution,
) -> String {
    match pt {
        dotnetdll::resolved::signature::ParameterType::Value(t) => format_method_type(t, res),
        dotnetdll::resolved::signature::ParameterType::Ref(t) => {
            format!("ref {}", format_method_type(t, res))
        }
        dotnetdll::resolved::signature::ParameterType::TypedReference => "TypedReference".into(),
    }
}

pub(in crate::assembly) fn format_method_ref_parent(
    parent: &MethodReferenceParent,
    res: &Resolution,
) -> String {
    match parent {
        MethodReferenceParent::Type(mt) => format_method_type(mt, res),
        MethodReferenceParent::Module(_) => String::new(),
        MethodReferenceParent::VarargMethod(_) => String::new(),
    }
}
