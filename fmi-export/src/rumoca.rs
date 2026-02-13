use std::{
    env, fs,
    path::{Path, PathBuf},
};

use thiserror::Error;

const RUST_FMI_TEMPLATE: &str = include_str!("../templates/rust-fmi.jinja");
const DEFAULT_OUTPUT_NAME: &str = "generated_fmu.rs";

#[derive(Debug, Error)]
pub enum RumocaError {
    #[error("missing OUT_DIR environment variable")]
    MissingOutDir,
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

pub type RumocaResult<T> = Result<T, RumocaError>;

/// Render the built-in rust-fmi template from a Modelica file.
pub fn render_modelica_to_rust(
    model_name: &str,
    model_path: impl AsRef<Path>,
) -> RumocaResult<String> {
    let model_path = model_path.as_ref();
    let model_path_str = model_path.to_string_lossy();

    let result = rumoca::Compiler::new()
        .model(model_name)
        .compile_file(&model_path_str)?;

    let rust_code = rumoca::dae::jinja::render_template_str(result.dae(), RUST_FMI_TEMPLATE)?;
    Ok(rust_code)
}

/// Render the built-in rust-fmi template from a Modelica file and write it to disk.
pub fn write_modelica_to_rust_file(
    model_name: &str,
    model_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> RumocaResult<()> {
    let rust_code = render_modelica_to_rust(model_name, model_path)?;
    fs::write(output_path, rust_code)?;
    Ok(())
}

/// Render the built-in rust-fmi template from a Modelica file into `$OUT_DIR/generated_fmu.rs`.
pub fn write_modelica_to_out_dir(
    model_name: &str,
    model_path: impl AsRef<Path>,
) -> RumocaResult<PathBuf> {
    let out_dir = env::var("OUT_DIR").map_err(|_| RumocaError::MissingOutDir)?;
    let output_path = PathBuf::from(out_dir).join(DEFAULT_OUTPUT_NAME);
    write_modelica_to_rust_file(model_name, model_path, &output_path)?;
    Ok(output_path)
}
