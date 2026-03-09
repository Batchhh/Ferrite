//! IL disassembler — emits ildasm-style text from parsed method bodies.

mod opcodes;

use crate::assembly::{Assembly, CustomAttribute, MethodDef, PeError, TypeDef, TypeKind};
use crate::decompiler::resolver::MetadataResolver;
use opcodes::{format_opcode, format_operand};

const INDENT: &str = "    ";

/// Disassemble a type to ildasm-style IL text.
pub fn disassemble_type_il(assembly: &Assembly, type_token: u32) -> Result<String, PeError> {
    let td = crate::decompiler::find_type_by_token(assembly, type_token)
        .ok_or_else(|| PeError::Parse(format!("TypeDef token 0x{:08X} not found", type_token)))?;
    let resolver = MetadataResolver::new(assembly);
    let mut out = String::with_capacity(4096);
    emit_type(&mut out, td, &resolver, "");
    Ok(out)
}

fn emit_type(out: &mut String, td: &TypeDef, resolver: &MetadataResolver, prefix: &str) {
    // Type header
    let vis = il_type_visibility(td.flags);
    let kind_kw = match td.kind {
        TypeKind::Interface => "interface ",
        _ => "",
    };
    let sealed = if td.flags & 0x100 != 0 { "sealed " } else { "" };
    let abstract_ = if td.flags & 0x80 != 0 {
        "abstract "
    } else {
        ""
    };

    out.push_str(&format!(
        "{prefix}.class {vis}{abstract_}{sealed}{kind_kw}{name}",
        name = td.name,
    ));

    if let Some(ref base) = td.base_type {
        if base != "System.Object" || td.kind == TypeKind::Class {
            out.push_str(&format!("\n{prefix}    extends {base}"));
        }
    }
    if !td.interfaces.is_empty() {
        out.push_str(&format!(
            "\n{prefix}    implements {}",
            td.interfaces.join(", ")
        ));
    }
    out.push('\n');
    out.push_str(prefix);
    out.push_str("{\n");

    // Attributes
    emit_attributes(out, &td.custom_attributes, &format!("{prefix}{INDENT}"));
    if !td.custom_attributes.is_empty() && (!td.fields.is_empty() || !td.methods.is_empty()) {
        out.push('\n');
    }

    // Fields
    for field in &td.fields {
        let fvis = il_field_visibility(field.flags);
        let static_ = if field.flags & 0x0010 != 0 {
            "static "
        } else {
            ""
        };
        out.push_str(&format!(
            "{prefix}{INDENT}.field {fvis}{static_}{ty} {name}\n",
            ty = field.field_type,
            name = field.name,
        ));
        emit_attributes(
            out,
            &field.custom_attributes,
            &format!("{prefix}{INDENT}{INDENT}"),
        );
    }
    if !td.fields.is_empty() && !td.methods.is_empty() {
        out.push('\n');
    }

    // Methods
    for (i, method) in td.methods.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        emit_method(out, method, resolver, &format!("{prefix}{INDENT}"));
    }

    // Nested types
    for nested in &td.nested_types {
        out.push('\n');
        emit_type(out, nested, resolver, &format!("{prefix}{INDENT}"));
    }

    out.push_str(prefix);
    out.push_str("}\n");
}

fn emit_method(out: &mut String, method: &MethodDef, resolver: &MetadataResolver, prefix: &str) {
    let vis = il_method_visibility(method.flags);
    let static_ = if method.flags & 0x0010 != 0 {
        "static "
    } else {
        "instance "
    };
    let virtual_ = if method.flags & 0x0040 != 0 {
        "virtual "
    } else {
        ""
    };
    let abstract_ = if method.flags & 0x0400 != 0 {
        "abstract "
    } else {
        ""
    };
    let hidebysig = if method.flags & 0x0080 != 0 {
        "hidebysig "
    } else {
        ""
    };
    let specialname = if method.flags & 0x0800 != 0 {
        "specialname "
    } else {
        ""
    };

    let params: Vec<String> = method
        .params
        .iter()
        .map(|p| format!("{} {}", p.type_name, p.name))
        .collect();

    out.push_str(&format!(
        "{prefix}.method {vis}{hidebysig}{specialname}{static_}{virtual_}{abstract_}{ret} {name}({params}) cil managed\n",
        ret = method.return_type,
        name = method.name,
        params = params.join(", "),
    ));
    out.push_str(prefix);
    out.push_str("{\n");

    // Method attributes
    emit_attributes(out, &method.custom_attributes, &format!("{prefix}{INDENT}"));

    if let Some(ref body) = method.parsed_body {
        let inner = format!("{prefix}{INDENT}");

        out.push_str(&format!("{inner}.maxstack {}\n", body.max_stack));

        if !body.locals.is_empty() {
            let locals: Vec<String> = body
                .locals
                .iter()
                .enumerate()
                .map(|(i, ty)| format!("{ty} V_{i}"))
                .collect();
            out.push_str(&format!("{inner}.locals init ({})\n", locals.join(", ")));
        }

        if body.max_stack > 0 || !body.locals.is_empty() {
            out.push('\n');
        }

        for instr in &body.instructions {
            let opname = format_opcode(&instr.opcode);
            let operand_str = format_operand(&instr.operand, resolver);
            if operand_str.is_empty() {
                out.push_str(&format!("{inner}IL_{:04x}: {opname}\n", instr.offset));
            } else {
                out.push_str(&format!(
                    "{inner}IL_{:04x}: {opname} {operand_str}\n",
                    instr.offset
                ));
            }
        }
    } else {
        out.push_str(&format!("{prefix}{INDENT}// No method body\n"));
    }

    out.push_str(prefix);
    out.push_str("}\n");
}

fn il_type_visibility(flags: u32) -> &'static str {
    match flags & 0x07 {
        0x00 => "",
        0x01 => "public ",
        0x02 => "nested public ",
        0x03 => "nested private ",
        0x04 => "nested family ",
        0x05 => "nested assembly ",
        0x06 => "nested famorassem ",
        0x07 => "nested famandassem ",
        _ => "",
    }
}

fn il_method_visibility(flags: u16) -> &'static str {
    match flags & 0x0007 {
        0x0001 => "private ",
        0x0002 => "famandassem ",
        0x0003 => "assembly ",
        0x0004 => "family ",
        0x0005 => "famorassem ",
        0x0006 => "public ",
        _ => "privatescope ",
    }
}

fn il_field_visibility(flags: u16) -> &'static str {
    match flags & 0x0007 {
        0x0001 => "private ",
        0x0002 => "famandassem ",
        0x0003 => "assembly ",
        0x0004 => "family ",
        0x0005 => "famorassem ",
        0x0006 => "public ",
        _ => "privatescope ",
    }
}

fn emit_attributes(out: &mut String, attrs: &[CustomAttribute], prefix: &str) {
    for attr in attrs {
        if attr.arguments.is_empty() {
            out.push_str(&format!(
                "{prefix}.custom instance void {name}::.ctor()\n",
                name = attr.name,
            ));
        } else {
            out.push_str(&format!(
                "{prefix}.custom instance void {name}::.ctor({args})\n",
                name = attr.name,
                args = attr.arguments.join(", "),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_attribute(name: &str, args: &[&str]) -> CustomAttribute {
        CustomAttribute {
            name: name.to_string(),
            arguments: args.iter().map(|s| s.to_string()).collect(),
        }
    }

    #[test]
    fn test_emit_attributes_no_args() {
        let attrs = vec![make_attribute("Serializable", &[])];
        let mut out = String::new();
        emit_attributes(&mut out, &attrs, "    ");
        assert_eq!(out, "    .custom instance void Serializable::.ctor()\n");
    }

    #[test]
    fn test_emit_attributes_with_args() {
        let attrs = vec![make_attribute("Obsolete", &["\"Use NewMethod\""])];
        let mut out = String::new();
        emit_attributes(&mut out, &attrs, "    ");
        assert_eq!(
            out,
            "    .custom instance void Obsolete::.ctor(\"Use NewMethod\")\n"
        );
    }

    #[test]
    fn test_emit_attributes_empty() {
        let attrs: Vec<CustomAttribute> = vec![];
        let mut out = String::new();
        emit_attributes(&mut out, &attrs, "    ");
        assert_eq!(out, "");
    }
}
