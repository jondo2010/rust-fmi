//! Extract static data from the compiled FMU library

use anyhow::{Context, Result};
use libloading::Library;
use std::path::Path;

use fmi::fmi3::{binding, schema};

const MODEL_METADATA_SYM: &[u8] = b"model_metadata";
const INSTANTIATION_TOKEN_SYM: &[u8] = b"FMI3_INSTANTIATION_TOKEN";
const SUPPORTS_ME_SYM: &[u8] = b"fmi3SupportsModelExchange";
const SUPPORTS_CS_SYM: &[u8] = b"fmi3SupportsCoSimulation";
const SUPPORTS_SE_SYM: &[u8] = b"fmi3SupportsScheduledExecution";

pub struct ModelData {
    pub model_variables: schema::ModelVariables,
    pub model_structure: schema::ModelStructure,
    pub instantiation_token: String,
    pub supports_model_exchange: bool,
    pub supports_co_simulation: bool,
    pub supports_scheduled_execution: bool,
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

            let supports_model_exchange = {
                let symbol = lib.get::<fn() -> binding::fmi3Boolean>(SUPPORTS_ME_SYM)?;
                symbol()
            };

            let supports_co_simulation = {
                let symbol = lib.get::<fn() -> binding::fmi3Boolean>(SUPPORTS_CS_SYM)?;
                symbol()
            };

            let supports_scheduled_execution = {
                let symbol = lib.get::<fn() -> binding::fmi3Boolean>(SUPPORTS_SE_SYM)?;
                symbol()
            };

            Ok(ModelData {
                model_variables,
                model_structure,
                instantiation_token,
                supports_model_exchange,
                supports_co_simulation,
                supports_scheduled_execution,
            })
        }
    }
}
