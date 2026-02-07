use std::{
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;
use fmi::fmi3::schema;

/// Create the FMU ZIP package
pub fn package_fmu(
    model_identifier: &str,
    model_description: schema::Fmi3ModelDescription,
    build_description: schema::Fmi3BuildDescription,
    terminals_and_icons: Option<schema::Fmi3TerminalsAndIcons>,
    fmu_path: &Path,
    dylibs: &[(Option<&'static platforms::Platform>, PathBuf)],
) -> anyhow::Result<()> {
    log::debug!("Creating FMU package at: {}", fmu_path.display());

    // Ensure the output directory exists
    if let Some(parent) = fmu_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
    }

    let mut zw = zip::ZipWriter::new(std::fs::File::create(&fmu_path)?);

    let binaries_path = PathBuf::from("binaries");

    zw.set_comment("Created by rust-fmi");

    // Write the modelDescription.xml file
    zw.start_file(
        "modelDescription.xml",
        zip::write::SimpleFileOptions::default(),
    )?;
    let xml = fmi::schema::serialize(&model_description, false)
        .context("Failed to serialize model description")?;
    zw.write_all(xml.as_bytes())?;

    // Write the buildDescription.xml file
    zw.start_file(
        "buildDescription.xml",
        zip::write::SimpleFileOptions::default(),
    )?;
    let build_xml = fmi::schema::serialize(&build_description, false)
        .context("Failed to serialize build description")?;
    zw.write_all(build_xml.as_bytes())?;

    if let Some(terminals) = terminals_and_icons {
        let terminals_dir = PathBuf::from("resources").join("terminalsAndIcons");
        zw.add_directory_from_path(&terminals_dir, zip::write::SimpleFileOptions::default())?;
        zw.start_file_from_path(
            terminals_dir.join("terminalsAndIcons.xml"),
            zip::write::SimpleFileOptions::default(),
        )?;
        let terminals_xml = fmi::schema::serialize(&terminals, false)
            .context("Failed to serialize terminals and icons")?;
        zw.write_all(terminals_xml.as_bytes())?;
    }

    for (platform, dylib_path) in dylibs {
        let (os, arch) = if let Some(p) = platform {
            (p.target_os.as_str(), p.target_arch.as_str())
        } else {
            (std::env::consts::OS, std::env::consts::ARCH)
        };
        let folder = fmi::fmi3::platform_folder(os, arch)?;
        let path = binaries_path.join(folder);

        // Create a filename for the archive entry consisting of the model identifier and the extension
        let filename = format!(
            "{}.{}",
            model_identifier,
            dylib_path
                .extension()
                .expect("Dylib must have an extension")
                .to_str()
                .unwrap()
        );

        zw.add_directory_from_path(&path, zip::write::SimpleFileOptions::default())?;
        zw.start_file_from_path(
            path.join(&filename),
            zip::write::SimpleFileOptions::default(),
        )?;

        let mut f = std::fs::File::open(dylib_path)?;
        std::io::copy(&mut f, &mut zw)?;
    }

    zw.finish()?;

    Ok(())
}
