use std::collections::HashMap;

mod attributes;
mod convert;
mod format;
mod format_types;
mod instructions;
mod method_body;
mod parse_types;
mod resolve;
mod tree;
mod types;

pub use types::*;

impl Assembly {
    /// Parse a .NET assembly from raw PE/CLR bytes.
    ///
    /// Resolves all type definitions, methods, fields, properties, and nested types.
    /// The resulting `Assembly` holds flat name tables used later for IL token resolution.
    pub fn parse(data: &[u8]) -> Result<Self, PeError> {
        use dotnetdll::prelude::*;

        let res = Resolution::parse(data, ReadOptions::default())
            .map_err(|e| PeError::Parse(format!("{}", e)))?;

        let mut interner = StringInterner::new();
        let num_types = res.type_definitions.len();
        let num_methods: usize = res.type_definitions.iter().map(|t| t.methods.len()).sum();
        let num_fields: usize = res.type_definitions.iter().map(|t| t.fields.len()).sum();

        let mut asm = Self {
            name: String::new(),
            version: String::new(),
            module_name: res.module.name.to_string(),
            target_framework: String::new(),
            types: Vec::with_capacity(num_types),
            assembly_refs: Vec::with_capacity(res.assembly_references.len()),
            type_def_names: Vec::with_capacity(num_types),
            method_def_names: Vec::with_capacity(num_methods),
            field_def_names: Vec::with_capacity(num_fields),
            type_ref_names: Vec::with_capacity(res.type_references.len()),
            member_ref_infos: Vec::with_capacity(res.method_references.len()),
            type_spec_names: Vec::new(),
            method_spec_infos: Vec::new(),
            method_def_call_infos: Vec::with_capacity(num_methods),
            us_heap_data: Vec::new(),
            user_strings: HashMap::new(),
            next_string_token: 1,
        };

        if let Some(ref asm_def) = res.assembly {
            asm.name = asm_def.name.to_string();
            let v = &asm_def.version;
            asm.version = format!("{}.{}.{}.{}", v.major, v.minor, v.build, v.revision);
        }

        for asmref in &res.assembly_references {
            asm.assembly_refs.push(AssemblyRef {
                name: asmref.name.to_string(),
                version: format!(
                    "{}.{}.{}.{}",
                    asmref.version.major,
                    asmref.version.minor,
                    asmref.version.build,
                    asmref.version.revision
                ),
            });
        }

        for typeref in &res.type_references {
            asm.type_ref_names.push(interner.intern(&typeref.name));
        }

        for methref in &res.method_references {
            let class_name = interner.intern_string(format_types::format_method_ref_parent(
                &methref.parent,
                &res,
            ));
            asm.member_ref_infos.push(MemberRefInfo {
                class_name,
                name: interner.intern(&methref.name),
                signature_blob: convert::encode_method_sig_blob(&methref.signature),
            });
        }

        let (type_defs_flat, type_encloser_map) =
            parse_types::parse_type_definitions(&res, &mut asm, &mut interner);

        // Extract target framework from assembly attributes
        if let Some(ref asm_def) = res.assembly {
            for attr in &asm_def.attributes {
                if let Some(name) = attributes::get_attribute_type_name(attr, &res) {
                    if name == "TargetFramework" {
                        // Decode the first constructor arg (the framework string)
                        use dotnetdll::resolved::types::AlwaysFailsResolver;
                        if let Ok(data) = attr.instantiation_data(&AlwaysFailsResolver, &res) {
                            if let Some(dotnetdll::resolved::attribute::FixedArg::String(Some(s))) =
                                data.constructor_args.first()
                            {
                                asm.target_framework = s.to_string();
                            }
                        }
                    }
                }
            }
        }

        asm.types = tree::build_type_tree(type_defs_flat, &type_encloser_map);

        Ok(asm)
    }
}

#[cfg(test)]
mod tests;
