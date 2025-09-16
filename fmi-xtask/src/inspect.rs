use anyhow::{Context, Result};
use clap::Args;

use crate::{extractor, fmu_builder::FmuBuilder, CommonArgs};

#[derive(Args)]
pub struct InspectArgs {
    #[command(flatten)]
    pub common: CommonArgs,
}

pub fn run(args: InspectArgs) -> Result<()> {
    inspect_package(
        &args.common.package,
        args.common.target.as_deref(),
        args.common.release,
    )
}

/// Inspect a package and pretty-print its extracted ModelData XML
fn inspect_package(package_name: &str, target: Option<&str>, release: bool) -> Result<()> {
    println!("=== Package Inspection: {} ===", package_name);

    // Create FmuBuilder to build the dylib
    let builder = FmuBuilder::new_for_package(package_name.to_string(), release)?;

    // Determine target
    let target = if let Some(target) = target {
        target.to_string()
    } else {
        // Get native target
        let output = std::process::Command::new("rustc")
            .args(["-vV"])
            .output()
            .context("Failed to get rustc info")?;

        let rustc_info = String::from_utf8(output.stdout)?;
        rustc_info
            .lines()
            .find(|line| line.starts_with("host: "))
            .and_then(|line| line.strip_prefix("host: "))
            .context("Could not determine host target")?
            .to_string()
    };

    println!("Target: {}", target);
    println!("Release: {}", release);

    // Build the dylib to get the path
    let dylib_path = builder.build_dylib(&target)?;
    println!("Built dylib: {}", dylib_path.display());

    // Extract the model data from dylib symbols
    let model_data = extractor::extract_model_data(&dylib_path)?;

    println!(
        "\nInstantiation Token {}\n---",
        model_data.instantiation_token
    );

    // Serialize back to XML with formatting
    let variables_xml = fmi::schema::serialize(&model_data.model_variables, true)?;
    let structure_xml = fmi::schema::serialize(&model_data.model_structure, true)?;

    println!("{variables_xml}\n---");
    println!("{structure_xml}");

    Ok(())
}
