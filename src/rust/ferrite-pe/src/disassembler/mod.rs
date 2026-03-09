//! IL disassembler — emits ildasm-style text from parsed method bodies.

mod opcodes;
mod visibility;

use crate::assembly::{Assembly, CustomAttribute, MethodDef, PeError, TypeDef, TypeKind};
use crate::decompiler::resolver::MetadataResolver;
use opcodes::{format_opcode, format_operand};
use visibility::{il_field_visibility, il_method_visibility, il_type_visibility};

#[cfg(test)]
mod tests;

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
    let inner = format!("{prefix}{INDENT}");

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
    emit_attributes(out, &td.custom_attributes, &inner);
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
            "{inner}.field {fvis}{static_}{ty} {name}\n",
            ty = field.field_type,
            name = field.name,
        ));
        emit_attributes(out, &field.custom_attributes, &format!("{inner}{INDENT}"));
    }
    if !td.fields.is_empty() && !td.methods.is_empty() {
        out.push('\n');
    }

    // TODO: emit properties and their attributes

    // Methods
    for (i, method) in td.methods.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        emit_method(out, method, resolver, &inner);
    }

    // Nested types
    for nested in &td.nested_types {
        out.push('\n');
        emit_type(out, nested, resolver, &inner);
    }

    out.push_str(prefix);
    out.push_str("}\n");
}

fn flag(flags: u16, mask: u16, label: &str) -> &str {
    if flags & mask != 0 {
        label
    } else {
        ""
    }
}

fn emit_method(out: &mut String, method: &MethodDef, resolver: &MetadataResolver, prefix: &str) {
    let vis = il_method_visibility(method.flags);
    let f = method.flags;
    let static_ = if f & 0x0010 != 0 {
        "static "
    } else {
        "instance "
    };
    let virtual_ = flag(f, 0x0040, "virtual ");
    let abstract_ = flag(f, 0x0400, "abstract ");
    let hidebysig = flag(f, 0x0080, "hidebysig ");
    let specialname = flag(f, 0x0800, "specialname ");

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

fn emit_attributes(out: &mut String, attrs: &[CustomAttribute], prefix: &str) {
    for attr in attrs {
        let args = if attr.arguments.is_empty() {
            String::new()
        } else {
            attr.arguments.join(", ")
        };
        out.push_str(&format!(
            "{prefix}.custom instance void {name}::.ctor({args})\n",
            name = attr.name,
        ));
    }
}
