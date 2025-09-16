//! Extract static data from the compiled FMU library

use anyhow::{Context, Result};
use libloading::Library;
use std::path::Path;

use fmi::fmi3::schema;

const MODEL_VARIABLES_SYM: &[u8] = b"FMI3_MODEL_VARIABLES";
const MODEL_STRUCTURE_SYM: &[u8] = b"FMI3_MODEL_STRUCTURE";
const INSTANTIATION_TOKEN_SYM: &[u8] = b"FMI3_INSTANTIATION_TOKEN";

pub struct ModelData {
    pub model_variables: schema::ModelVariables,
    pub model_structure: schema::ModelStructure,
    pub instantiation_token: String,
}

impl ModelData {
    /// Extract model XML from a compiled FMU dylib
    pub fn new_from_dylib(dylib_path: &Path) -> Result<Self> {
        unsafe {
            // Load the dynamic library
            let lib = Library::new(dylib_path)
                .with_context(|| format!("Failed to load dylib: {}", dylib_path.display()))?;

            let variables_xml = {
                let symbol = lib.get::<*const &str>(MODEL_VARIABLES_SYM)?;
                let string: &str = **symbol;
                if string.starts_with("<ModelVariables") {
                    string
                } else {
                    anyhow::bail!("Symbol doesn't contain ModelVariables xml")
                }
            };

            let structure_xml = {
                let symbol = lib.get::<*const &str>(MODEL_STRUCTURE_SYM)?;
                let string: &str = **symbol;
                if string.starts_with("<ModelStructure") {
                    string
                } else {
                    anyhow::bail!("Symbol doesn't contain ModelStructure xml")
                }
            };

            let instantiation_token = {
                let symbol = lib.get::<*const &str>(INSTANTIATION_TOKEN_SYM)?;
                let string: &str = **symbol;
                if !string.is_empty() {
                    string.to_string()
                } else {
                    anyhow::bail!("Symbol doesn't contain InstantiationToken")
                }
            };

            // Parse the XML strings into structs
            let model_variables: schema::ModelVariables =
                fmi::schema::deserialize(variables_xml)
                    .context("Failed to parse ModelVariables XML")?;

            let model_structure: schema::ModelStructure =
                fmi::schema::deserialize(structure_xml)
                    .context("Failed to parse ModelStructure XML")?;

            Ok(ModelData {
                model_variables,
                model_structure,
                instantiation_token,
            })
        }
    }
}
