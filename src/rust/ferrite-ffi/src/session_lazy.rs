use crate::convert::*;
use crate::types::*;
use crate::DecompilerSession;

#[uniffi::export]
impl DecompilerSession {
    /// Load a .NET assembly and return a lightweight summary (no type details).
    pub fn load_assembly_lazy(&self, path: String) -> Result<AssemblySummary, FerriteError> {
        let file = std::fs::File::open(&path).map_err(|e| FerriteError::IoError {
            message: format!("{}: {}", path, e),
        })?;
        let data = unsafe { memmap2::Mmap::map(&file) }.map_err(|e| FerriteError::IoError {
            message: format!("{}: {}", path, e),
        })?;

        let assembly =
            ferrite_pe::assembly::Assembly::parse(&data).map_err(|e| FerriteError::ParseError {
                message: e.to_string(),
            })?;

        let mut id_counter = self.next_id.lock();
        let id = format!("asm_{}", *id_counter);
        *id_counter += 1;
        drop(id_counter);

        let info = convert_assembly(&assembly);
        let summary = convert_assembly_summary(&assembly, &id, &path);

        let loaded = crate::LoadedAssembly {
            id,
            file_path: path,
            info,
            assembly,
        };

        self.assemblies.write().push(loaded);
        Ok(summary)
    }

    /// Return lightweight type summaries for all types in a namespace.
    pub fn get_namespace_types(
        &self,
        assembly_id: String,
        namespace: String,
    ) -> Result<Vec<TypeSummary>, FerriteError> {
        let assemblies = self.assemblies.read();
        let loaded = assemblies
            .iter()
            .find(|a| a.id == assembly_id)
            .ok_or_else(|| FerriteError::NotFound {
                message: format!("Assembly '{}' not found", assembly_id),
            })?;
        Ok(convert_namespace_types(&loaded.assembly, &namespace))
    }

    /// Return full type details for a single type by token.
    pub fn get_type_details(
        &self,
        assembly_id: String,
        type_token: u32,
    ) -> Result<TypeInfo, FerriteError> {
        let assemblies = self.assemblies.read();
        let loaded = assemblies
            .iter()
            .find(|a| a.id == assembly_id)
            .ok_or_else(|| FerriteError::NotFound {
                message: format!("Assembly '{}' not found", assembly_id),
            })?;

        find_type_and_convert(&loaded.assembly, type_token).ok_or_else(|| FerriteError::NotFound {
            message: format!("Type token 0x{:08X} not found", type_token),
        })
    }

    /// Return all searchable items (types + members) for building the search index.
    pub fn get_searchable_items(
        &self,
        assembly_id: String,
    ) -> Result<Vec<SearchableItem>, FerriteError> {
        let assemblies = self.assemblies.read();
        let loaded = assemblies
            .iter()
            .find(|a| a.id == assembly_id)
            .ok_or_else(|| FerriteError::NotFound {
                message: format!("Assembly '{}' not found", assembly_id),
            })?;
        Ok(convert_searchable_items(&loaded.assembly))
    }
}
