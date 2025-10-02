//! Extract static data from the compiled FMU library

use anyhow::{Context, Result};
use libloading::Library;
use std::path::Path;

use fmi::fmi3::schema;

const MODEL_METADATA_SYM: &[u8] = b"model_metadata";
const INSTANTIATION_TOKEN_SYM: &[u8] = b"FMI3_INSTANTIATION_TOKEN";

pub struct ModelData {
    pub model_variables: schema::ModelVariables,
    pub model_structure: schema::ModelStructure,
    pub instantiation_token: String,
}

impl ModelData {
    /// Extract model metadata from a compiled FMU dylib
    pub fn new_from_dylib(dylib_path: &Path) -> Result<Self> {
        unsafe {
            // Load the dynamic library
            let lib = Library::new(dylib_path)
                .with_context(|| format!("Failed to load dylib: {}", dylib_path.display()))?;

            // Call the new Rust ABI model_metadata() function
            let (model_variables, model_structure) = {
                let symbol = lib.get::<fn() -> (schema::ModelVariables, schema::ModelStructure)>(
                    MODEL_METADATA_SYM,
                )?;
                symbol()
            };

            log::info!(
                "Extracted {} model variables from dylib.",
                model_variables.len()
            );

            let instantiation_token = {
                let symbol = lib.get::<*const &str>(INSTANTIATION_TOKEN_SYM)?;
                let string: &str = **symbol;
                if !string.is_empty() {
                    string.to_string()
                } else {
                    anyhow::bail!("Symbol doesn't contain InstantiationToken")
                }
            };

            Ok(ModelData {
                model_variables,
                model_structure,
                instantiation_token,
            })
        }
    }
}
