//! IL disassembler — emits ildasm-style text from parsed method bodies.

mod exception_handlers;
mod opcodes;
mod visibility;

use crate::assembly::{
    Assembly, CustomAttribute, EventDef, MethodDef, PeError, PropertyDef, TypeDef, TypeKind,
};
use crate::decompiler::resolver::MetadataResolver;
use exception_handlers::emit_exception_handlers;
use opcodes::{format_opcode, format_operand};
use visibility::{il_field_visibility, il_method_visibility, il_type_visibility};

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

    // Properties
    for prop in &td.properties {
        emit_property(out, prop, td, &inner);
    }
    if !td.properties.is_empty() && !td.methods.is_empty() {
        out.push('\n');
    }

    // Events
    for event in &td.events {
        emit_event(out, event, td, &inner);
    }
    if !td.events.is_empty() && !td.methods.is_empty() {
        out.push('\n');
    }

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

        emit_exception_handlers(out, &body.exception_handlers, resolver, &inner);

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

fn emit_property(out: &mut String, prop: &PropertyDef, td: &TypeDef, prefix: &str) {
    let is_instance = prop
        .getter_token
        .or(prop.setter_token)
        .and_then(|tok| td.methods.iter().find(|m| m.token == tok))
        .map(|m| m.flags & 0x0010 == 0)
        .unwrap_or(true);

    let instance_str = if is_instance { "instance " } else { "" };

    out.push_str(&format!(
        "{prefix}.property {instance_str}{ty} {name}()\n",
        ty = prop.property_type,
        name = prop.name,
    ));
    out.push_str(prefix);
    out.push_str("{\n");

    let inner = format!("{prefix}{INDENT}");
    emit_attributes(out, &prop.custom_attributes, &inner);

    if let Some(getter_tok) = prop.getter_token {
        if let Some(getter) = td.methods.iter().find(|m| m.token == getter_tok) {
            let vis = il_method_visibility(getter.flags);
            let static_ = if getter.flags & 0x0010 != 0 {
                ""
            } else {
                "instance "
            };
            let params: Vec<String> = getter.params.iter().map(|p| p.type_name.clone()).collect();
            out.push_str(&format!(
                "{inner}.get {vis}{static_}{ret} {cls}::{name}({params})\n",
                ret = getter.return_type,
                cls = td.name,
                name = getter.name,
                params = params.join(", "),
            ));
        }
    }
    if let Some(setter_tok) = prop.setter_token {
        if let Some(setter) = td.methods.iter().find(|m| m.token == setter_tok) {
            let vis = il_method_visibility(setter.flags);
            let static_ = if setter.flags & 0x0010 != 0 {
                ""
            } else {
                "instance "
            };
            let params: Vec<String> = setter.params.iter().map(|p| p.type_name.clone()).collect();
            out.push_str(&format!(
                "{inner}.set {vis}{static_}{ret} {cls}::{name}({params})\n",
                ret = setter.return_type,
                cls = td.name,
                name = setter.name,
                params = params.join(", "),
            ));
        }
    }

    out.push_str(prefix);
    out.push_str("}\n");
}

fn emit_event(out: &mut String, event: &EventDef, td: &TypeDef, prefix: &str) {
    out.push_str(&format!(
        "{prefix}.event {ty} {name}\n",
        ty = event.event_type,
        name = event.name,
    ));
    out.push_str(prefix);
    out.push_str("{\n");

    let inner = format!("{prefix}{INDENT}");
    emit_attributes(out, &event.custom_attributes, &inner);

    if let Some(add_tok) = event.add_token {
        if let Some(adder) = td.methods.iter().find(|m| m.token == add_tok) {
            let vis = il_method_visibility(adder.flags);
            let static_ = if adder.flags & 0x0010 != 0 {
                ""
            } else {
                "instance "
            };
            let params: Vec<String> = adder.params.iter().map(|p| p.type_name.clone()).collect();
            out.push_str(&format!(
                "{inner}.addon {vis}{static_}{ret} {cls}::{name}({params})\n",
                ret = adder.return_type,
                cls = td.name,
                name = adder.name,
                params = params.join(", "),
            ));
        }
    }
    if let Some(remove_tok) = event.remove_token {
        if let Some(remover) = td.methods.iter().find(|m| m.token == remove_tok) {
            let vis = il_method_visibility(remover.flags);
            let static_ = if remover.flags & 0x0010 != 0 {
                ""
            } else {
                "instance "
            };
            let params: Vec<String> = remover.params.iter().map(|p| p.type_name.clone()).collect();
            out.push_str(&format!(
                "{inner}.removeon {vis}{static_}{ret} {cls}::{name}({params})\n",
                ret = remover.return_type,
                cls = td.name,
                name = remover.name,
                params = params.join(", "),
            ));
        }
    }
    if let Some(raise_tok) = event.raise_token {
        if let Some(raiser) = td.methods.iter().find(|m| m.token == raise_tok) {
            let vis = il_method_visibility(raiser.flags);
            let static_ = if raiser.flags & 0x0010 != 0 {
                ""
            } else {
                "instance "
            };
            let params: Vec<String> = raiser.params.iter().map(|p| p.type_name.clone()).collect();
            out.push_str(&format!(
                "{inner}.fire {vis}{static_}{ret} {cls}::{name}({params})\n",
                ret = raiser.return_type,
                cls = td.name,
                name = raiser.name,
                params = params.join(", "),
            ));
        }
    }

    out.push_str(prefix);
    out.push_str("}\n");
}

fn emit_attributes(out: &mut String, attrs: &[CustomAttribute], prefix: &str) {
    for attr in attrs {
        let args = attr.arguments.join(", ");
        out.push_str(&format!(
            "{prefix}.custom instance void {}::.ctor({args})\n",
            attr.name,
        ));
    }
}
