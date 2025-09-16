//! Implements the `bundle` command to create an FMU package.

use crate::metadata::MetadataBuilder;

pub fn bundle(
    package: &Option<String>,
    target: &Option<Vec<String>>,
    release: bool,
) -> anyhow::Result<()> {
    let MetadataBuilder {
        package,
        model_identifier,
        target_dir,
    } = MetadataBuilder::new(package.as_deref())?;

    let target_platforms = target
        .as_ref()
        .map(|ts| {
            ts.iter()
                .map(|t| {
                    platforms::Platform::find(t)
                        .ok_or_else(|| anyhow::anyhow!("Unknown target platform: {}", t))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;

    // Build the cdylib for the specified targets or native if none specified
    let cdylibs = crate::builder::build_lib(&package.id, &target_platforms, release)?;

    let model_data = crate::extractor::ModelData::new_from_dylib(&cdylibs[0].1)?;
    let model_description =
        crate::metadata::create_model_description(&model_identifier, &package, model_data)?;
    let build_description = crate::metadata::create_build_description(&model_identifier)?;

    // Create FMU package
    let fmu_path = target_dir
        .join("fmu")
        .join(format!("{model_identifier}.fmu"));
    log::info!("Creating FMU package at: {}", fmu_path.display());

    crate::packager::package_fmu(
        &model_identifier,
        model_description,
        build_description,
        &fmu_path,
        &cdylibs,
    )?;

    Ok(())
}
