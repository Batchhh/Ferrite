//! Metadata token resolver — converts IL metadata tokens to human-readable names.

use crate::assembly::Assembly;
use crate::il::Operand;

/// Resolves metadata tokens embedded in IL instructions to human-readable names.
pub struct MetadataResolver<'a> {
    pub assembly: &'a Assembly,
}

impl<'a> MetadataResolver<'a> {
    pub fn new(assembly: &'a Assembly) -> Self {
        Self { assembly }
    }

    /// Resolve a metadata token to a human-readable name.
    ///
    /// Token format: high byte = table ID, low 3 bytes = row index (1-based).
    pub fn resolve_token(&self, token: u32) -> String {
        let table = (token >> 24) as u8;
        let row = (token & 0x00FFFFFF) as usize;
        match table {
            0x01 => self
                .assembly
                .type_ref_names
                .get(row.wrapping_sub(1))
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("TypeRef_{}", row)),
            0x02 => self
                .assembly
                .type_def_names
                .get(row.wrapping_sub(1))
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("TypeDef_{}", row)),
            0x04 => self
                .assembly
                .field_def_names
                .get(row.wrapping_sub(1))
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("Field_{}", row)),
            0x06 => self
                .assembly
                .method_def_names
                .get(row.wrapping_sub(1))
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("Method_{}", row)),
            0x0A => self.resolve_member_ref(row),
            0x1B => self
                .assembly
                .type_spec_names
                .get(row.wrapping_sub(1))
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("TypeSpec_{}", row)),
            0x2B => self.resolve_method_spec(row),
            0x70 => {
                // User string token — look up in pre-resolved map
                let s = self.resolve_user_string(token);
                format!("\"{}\"", s)
            }
            _ => format!("/*0x{:08X}*/", token),
        }
    }

    /// Resolve a token or a ResolvedName operand.
    #[allow(dead_code)]
    pub fn resolve_operand(&self, operand: &Operand) -> String {
        match operand {
            Operand::Token(tok) => self.resolve_token(*tok),
            Operand::ResolvedName(name) => name.clone(),
            _ => String::new(),
        }
    }

    /// Resolve a MethodSpec token to "MethodName<T1, T2>" format.
    fn resolve_method_spec(&self, row: usize) -> String {
        if let Some(info) = self.assembly.method_spec_infos.get(row.wrapping_sub(1)) {
            let base_name = self.resolve_token(info.base_method_token);
            if info.generic_args.is_empty() {
                base_name
            } else {
                let clean = if let Some(idx) = base_name.find('`') {
                    &base_name[..idx]
                } else {
                    &base_name
                };
                format!("{}<{}>", clean, info.generic_args.join(", "))
            }
        } else {
            format!("MethodSpec_{}", row)
        }
    }

    /// Resolve a MemberRef token to "ClassName::MemberName".
    fn resolve_member_ref(&self, row: usize) -> String {
        if let Some(info) = self.assembly.member_ref_infos.get(row.wrapping_sub(1)) {
            if info.class_name.is_empty() {
                info.name.to_string()
            } else {
                format!("{}::{}", info.class_name, info.name)
            }
        } else {
            format!("MemberRef_{}", row)
        }
    }

    /// Resolve a user string token.
    pub fn resolve_user_string(&self, token: u32) -> String {
        self.assembly
            .user_strings
            .get(&token)
            .cloned()
            .unwrap_or_default()
    }

    /// Get call info for a method token: (name, param_count, has_return, is_static).
    pub fn resolve_call_info(&self, token: u32) -> (String, usize, bool, bool) {
        let table = (token >> 24) as u8;
        let row = (token & 0x00FFFFFF) as usize;

        match table {
            0x06 => {
                let name = self
                    .assembly
                    .method_def_names
                    .get(row.wrapping_sub(1))
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("Method_{}", row));
                let (param_count, has_return, is_static) =
                    self.assembly.get_method_def_call_info(row);
                (name, param_count, has_return, is_static)
            }
            0x0A => {
                if let Some(info) = self.assembly.member_ref_infos.get(row.wrapping_sub(1)) {
                    let name = if info.class_name.is_empty() {
                        info.name.to_string()
                    } else {
                        format!("{}::{}", info.class_name, info.name)
                    };
                    let (param_count, has_return, is_static) =
                        parse_method_sig_call_info(&info.signature_blob);
                    (name, param_count, has_return, is_static)
                } else {
                    (format!("MemberRef_{}", row), 0, false, true)
                }
            }
            0x2B => {
                if let Some(info) = self.assembly.method_spec_infos.get(row.wrapping_sub(1)) {
                    let (base_name, param_count, has_return, is_static) =
                        self.resolve_call_info(info.base_method_token);
                    let name = self.resolve_method_spec(row);
                    let final_name = if let Some(pos) = base_name.rfind("::") {
                        let qualifier = &base_name[..pos];
                        let method_part = if let Some(mpos) = name.rfind("::") {
                            &name[mpos + 2..]
                        } else {
                            &name
                        };
                        format!("{}::{}", qualifier, method_part)
                    } else {
                        name
                    };
                    (final_name, param_count, has_return, is_static)
                } else {
                    (format!("MethodSpec_{}", row), 0, false, true)
                }
            }
            _ => (format!("/*0x{:08X}*/", token), 0, false, true),
        }
    }
}

/// Read a compressed unsigned integer from raw heap data.
fn read_compressed_u32_from_heap(data: &[u8], pos: &mut usize) -> u32 {
    if *pos >= data.len() {
        return 0;
    }
    let b0 = data[*pos] as u32;
    if b0 & 0x80 == 0 {
        *pos += 1;
        b0
    } else if b0 & 0xC0 == 0x80 {
        if *pos + 1 >= data.len() {
            return 0;
        }
        let b1 = data[*pos + 1] as u32;
        *pos += 2;
        ((b0 & 0x3F) << 8) | b1
    } else {
        if *pos + 3 >= data.len() {
            return 0;
        }
        let b1 = data[*pos + 1] as u32;
        let b2 = data[*pos + 2] as u32;
        let b3 = data[*pos + 3] as u32;
        *pos += 4;
        ((b0 & 0x1F) << 24) | (b1 << 16) | (b2 << 8) | b3
    }
}

/// Parse a method signature blob to extract (param_count, has_return_value, is_static).
fn parse_method_sig_call_info(blob: &[u8]) -> (usize, bool, bool) {
    if blob.is_empty() {
        return (0, false, true);
    }
    let calling_conv = blob[0];
    let is_static = (calling_conv & 0x20) == 0;
    let mut pos = 1usize;

    if (calling_conv & 0x10) != 0 {
        let _ = read_compressed_u32_from_heap(blob, &mut pos);
    }

    let param_count = read_compressed_u32_from_heap(blob, &mut pos) as usize;
    let has_return = pos < blob.len() && blob[pos] != 0x01;

    (param_count, has_return, is_static)
}
