use std::sync::{Arc, Mutex, RwLock};

use ferrite_pe::assembly::Assembly;

pub(crate) mod types;
pub use types::*;

mod convert;
use convert::*;

uniffi::setup_scaffolding!();

/// In-memory cache of a loaded assembly: raw parsed form + pre-converted FFI metadata.
struct LoadedAssembly {
    id: String,
    file_path: String,
    info: AssemblyInfo,
    assembly: Assembly,
}

/// Manages a collection of loaded .NET assemblies for the decompiler UI.
///
/// Exported via UniFFI as an `Arc`-wrapped object; safe to share across threads.
#[derive(uniffi::Object)]
pub struct DecompilerSession {
    assemblies: RwLock<Vec<LoadedAssembly>>,
    next_id: Mutex<u64>,
}

#[uniffi::export]
impl DecompilerSession {
    #[uniffi::constructor]
    pub fn new() -> Arc<Self> {
        Arc::new(DecompilerSession {
            assemblies: RwLock::new(Vec::new()),
            next_id: Mutex::new(1),
        })
    }

    /// Load a .NET assembly from `path` and return the full entry (id + metadata).
    ///
    /// This avoids a separate `get_assembly_info` call and its deep clone.
    pub fn load_assembly(&self, path: String) -> Result<LoadedAssemblyEntry, FerriteError> {
        let file = std::fs::File::open(&path).map_err(|e| FerriteError::IoError {
            message: format!("{}: {}", path, e),
        })?;
        let data = unsafe { memmap2::Mmap::map(&file) }.map_err(|e| FerriteError::IoError {
            message: format!("{}: {}", path, e),
        })?;

        let assembly = Assembly::parse(&data).map_err(|e| FerriteError::ParseError {
            message: e.to_string(),
        })?;

        let info = convert_assembly(&assembly);

        let mut id_counter = self.next_id.lock().unwrap();
        let id = format!("asm_{}", *id_counter);
        *id_counter += 1;
        drop(id_counter);

        let entry = LoadedAssemblyEntry {
            id: id.clone(),
            file_path: path.clone(),
            info: info.clone(),
        };

        let loaded = LoadedAssembly {
            id,
            file_path: path,
            info,
            assembly,
        };

        self.assemblies.write().unwrap().push(loaded);
        Ok(entry)
    }

    /// Unload the assembly identified by `id`.
    pub fn remove_assembly(&self, id: String) -> Result<(), FerriteError> {
        let mut assemblies = self.assemblies.write().unwrap();
        let idx =
            assemblies
                .iter()
                .position(|a| a.id == id)
                .ok_or_else(|| FerriteError::NotFound {
                    message: format!("Assembly '{}' not found", id),
                })?;
        assemblies.remove(idx);
        Ok(())
    }

    /// Return a list of all currently loaded assemblies.
    pub fn get_assemblies(&self) -> Vec<LoadedAssemblyEntry> {
        self.assemblies
            .read()
            .unwrap()
            .iter()
            .map(|a| LoadedAssemblyEntry {
                id: a.id.clone(),
                file_path: a.file_path.clone(),
                info: a.info.clone(),
            })
            .collect()
    }

    /// Return full metadata for a single assembly.
    pub fn get_assembly_info(&self, id: String) -> Result<AssemblyInfo, FerriteError> {
        let assemblies = self.assemblies.read().unwrap();
        assemblies
            .iter()
            .find(|a| a.id == id)
            .map(|a| a.info.clone())
            .ok_or_else(|| FerriteError::NotFound {
                message: format!("Assembly '{}' not found", id),
            })
    }

    /// Decompile a type to C# source. `type_token` is a TypeDef token (0x02XXXXXX).
    pub fn decompile_type(
        &self,
        assembly_id: String,
        type_token: u32,
    ) -> Result<String, FerriteError> {
        let assemblies = self.assemblies.read().unwrap();
        let loaded = assemblies
            .iter()
            .find(|a| a.id == assembly_id)
            .ok_or_else(|| FerriteError::NotFound {
                message: format!("Assembly '{}' not found", assembly_id),
            })?;

        ferrite_pe::decompiler::decompile_type(&loaded.assembly, type_token).map_err(|e| {
            FerriteError::ParseError {
                message: e.to_string(),
            }
        })
    }

    // --- Lazy loading endpoints ---

    /// Load a .NET assembly and return a lightweight summary (no type details).
    pub fn load_assembly_lazy(&self, path: String) -> Result<AssemblySummary, FerriteError> {
        let file = std::fs::File::open(&path).map_err(|e| FerriteError::IoError {
            message: format!("{}: {}", path, e),
        })?;
        let data = unsafe { memmap2::Mmap::map(&file) }.map_err(|e| FerriteError::IoError {
            message: format!("{}: {}", path, e),
        })?;

        let assembly = Assembly::parse(&data).map_err(|e| FerriteError::ParseError {
            message: e.to_string(),
        })?;

        let mut id_counter = self.next_id.lock().unwrap();
        let id = format!("asm_{}", *id_counter);
        *id_counter += 1;
        drop(id_counter);

        let info = convert_assembly(&assembly);
        let summary = convert_assembly_summary(&assembly, &id, &path);

        let loaded = LoadedAssembly {
            id,
            file_path: path,
            info,
            assembly,
        };

        self.assemblies.write().unwrap().push(loaded);
        Ok(summary)
    }

    /// Return lightweight type summaries for all types in a namespace.
    pub fn get_namespace_types(
        &self,
        assembly_id: String,
        namespace: String,
    ) -> Result<Vec<TypeSummary>, FerriteError> {
        let assemblies = self.assemblies.read().unwrap();
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
        let assemblies = self.assemblies.read().unwrap();
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
        let assemblies = self.assemblies.read().unwrap();
        let loaded = assemblies
            .iter()
            .find(|a| a.id == assembly_id)
            .ok_or_else(|| FerriteError::NotFound {
                message: format!("Assembly '{}' not found", assembly_id),
            })?;
        Ok(convert_searchable_items(&loaded.assembly))
    }
}
