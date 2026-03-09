use std::sync::Arc;

use parking_lot::{Mutex, RwLock};

use ferrite_pe::assembly::Assembly;

pub(crate) mod types;
pub use types::*;

mod convert;
use convert::*;

mod session_lazy;

uniffi::setup_scaffolding!();

/// In-memory cache of a loaded assembly: raw parsed form + pre-converted FFI metadata.
pub(crate) struct LoadedAssembly {
    pub(crate) id: String,
    pub(crate) file_path: String,
    pub(crate) info: AssemblyInfo,
    pub(crate) assembly: Assembly,
}

/// Manages a collection of loaded .NET assemblies for the decompiler UI.
///
/// Exported via UniFFI as an `Arc`-wrapped object; safe to share across threads.
#[derive(uniffi::Object)]
pub struct DecompilerSession {
    pub(crate) assemblies: RwLock<Vec<LoadedAssembly>>,
    pub(crate) next_id: Mutex<u64>,
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

        let mut id_counter = self.next_id.lock();
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

        self.assemblies.write().push(loaded);
        Ok(entry)
    }

    /// Unload the assembly identified by `id`.
    pub fn remove_assembly(&self, id: String) -> Result<(), FerriteError> {
        let mut assemblies = self.assemblies.write();
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
        let assemblies = self.assemblies.read();
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
        let assemblies = self.assemblies.read();
        let loaded = assemblies
            .iter()
            .find(|a| a.id == assembly_id)
            .ok_or_else(|| FerriteError::NotFound {
                message: format!("Assembly '{}' not found", assembly_id),
            })?;

        ferrite_pe::decompiler::decompile_type(&loaded.assembly, type_token).map_err(|e| {
            FerriteError::DecompilationFailed {
                message: e.to_string(),
            }
        })
    }

    /// Disassemble a type to ildasm-style IL. `type_token` is a TypeDef token (0x02XXXXXX).
    pub fn disassemble_type_il(
        &self,
        assembly_id: String,
        type_token: u32,
    ) -> Result<String, FerriteError> {
        let assemblies = self.assemblies.read();
        let loaded = assemblies
            .iter()
            .find(|a| a.id == assembly_id)
            .ok_or_else(|| FerriteError::NotFound {
                message: format!("Assembly '{}' not found", assembly_id),
            })?;

        ferrite_pe::disassembler::disassemble_type_il(&loaded.assembly, type_token).map_err(|e| {
            FerriteError::DecompilationFailed {
                message: e.to_string(),
            }
        })
    }
}
