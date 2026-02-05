use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use zip::ZipArchive;

use crate::InspectFormat;

pub fn inspect(fmu_path: &Path, format: InspectFormat) -> Result<()> {
    let file = File::open(fmu_path)
        .with_context(|| format!("Failed to open FMU at {}", fmu_path.display()))?;
    let mut zip = ZipArchive::new(file)
        .with_context(|| format!("Failed to read FMU zip: {}", fmu_path.display()))?;

    let model_xml = read_zip_entry(&mut zip, "modelDescription.xml")?
        .context("FMU is missing modelDescription.xml")?;

    match format {
        InspectFormat::ModelDescription => {
            let pretty = pretty_model_description(&model_xml)?;
            println!("{pretty}");
        }
        InspectFormat::Debug => {
            println!("FMU: {}", fmu_path.display());
            println!("Entries:");
            for i in 0..zip.len() {
                let file = zip.by_index(i)?;
                println!(
                    "  - {} ({} bytes)",
                    file.name(),
                    file.size()
                );
            }

            if let Some(build_xml) = read_zip_entry(&mut zip, "buildDescription.xml")? {
                match pretty_build_description(&build_xml) {
                    Ok(pretty) => {
                        println!("\nBuildDescription (parsed):\n{pretty}");
                    }
                    Err(err) => {
                        println!("\nBuildDescription (raw):\n{build_xml}");
                        println!("BuildDescription parse error: {err}");
                    }
                }
            } else {
                println!("\nBuildDescription: (not present)");
            }

            println!("\nModelDescription: use --format model-description to view XML.");
        }
    }

    Ok(())
}

fn read_zip_entry(
    zip: &mut ZipArchive<File>,
    name: &str,
) -> Result<Option<String>> {
    match zip.by_name(name) {
        Ok(mut file) => {
            let mut contents = String::new();
            std::io::Read::read_to_string(&mut file, &mut contents)?;
            Ok(Some(contents))
        }
        Err(zip::result::ZipError::FileNotFound) => Ok(None),
        Err(err) => Err(err.into()),
    }
}

fn pretty_model_description(xml: &str) -> Result<String> {
    if let Ok(model) = fmi_schema::deserialize::<fmi_schema::fmi3::Fmi3ModelDescription>(xml) {
        return Ok(fmi_schema::serialize(&model, true)?);
    }

    let model = fmi_schema::deserialize::<fmi_schema::fmi2::Fmi2ModelDescription>(xml)?;
    Ok(fmi_schema::serialize(&model, true)?)
}

fn pretty_build_description(xml: &str) -> Result<String> {
    let build = fmi_schema::deserialize::<fmi_schema::fmi3::Fmi3BuildDescription>(xml)?;
    Ok(fmi_schema::serialize(&build, true)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use zip::ZipWriter;

    #[test]
    fn read_zip_entry_returns_none_when_missing() {
        let temp = NamedTempFile::new().expect("temp file");
        let zip = ZipWriter::new(temp.reopen().expect("reopen"));
        zip.finish().expect("finish zip");

        let mut archive = ZipArchive::new(temp.reopen().expect("reopen")).expect("open zip");
        let entry = read_zip_entry(&mut archive, "missing.txt").expect("read entry");
        assert!(entry.is_none());
    }
}
