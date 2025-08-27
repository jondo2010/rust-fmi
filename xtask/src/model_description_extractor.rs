use anyhow::{anyhow, Context, Result};
use libloading::Library;
use std::path::Path;

/// This is the symbol that should be exported in the FMU shared library.
const MODEL_DESCRIPTION_SYMBOL: &[u8] = b"FMI3_MODEL_DESCRIPTION";

/// Extract model description XML from a compiled FMU dylib
pub fn extract_model_description(dylib_path: &Path) -> Result<String> {
    unsafe {
        // Load the dynamic library
        let lib = Library::new(dylib_path)
            .with_context(|| format!("Failed to load dylib: {}", dylib_path.display()))?;

        let symbol = lib.get::<*const &str>(MODEL_DESCRIPTION_SYMBOL)?;
        let string: &str = **symbol;
        if string.starts_with("<?xml") && string.contains("fmiModelDescription") {
            // Return an owned String to avoid lifetime issues
            Ok(string.to_string())
        } else {
            Err(anyhow!("Symbol doesn't contain fmiModelDescription xml"))
        }
    }
}
