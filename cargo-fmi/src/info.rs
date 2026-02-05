use anyhow::Result;

use crate::metadata::MetadataBuilder;

pub fn info(package: &Option<String>, target: &Option<Vec<String>>, release: bool) -> Result<()> {
    let MetadataBuilder {
        package,
        model_identifier,
        ..
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

    let cdylibs = crate::builder::build_lib(&package.id, &target_platforms, release)?;
    let model_data = crate::extractor::ModelData::new_from_dylib(&cdylibs[0].1)?;
    let model_description =
        crate::metadata::create_model_description(&model_identifier, &package, model_data)?;

    println!("{:#?}", model_description);

    Ok(())
}
