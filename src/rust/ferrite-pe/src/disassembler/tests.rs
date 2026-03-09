//! Tests for the IL disassembler.

use super::*;
use crate::assembly::CustomAttribute;

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
