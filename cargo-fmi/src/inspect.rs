use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use fmi::schema::{self, traits::FmiModelDescription, MajorVersion};
use fmi::traits::FmiImport;

use crate::InspectFormat;

pub fn inspect(fmu_path: &Path, format: InspectFormat) -> Result<()> {
    let min_desc = fmi::import::peek_descr_path(fmu_path)
        .with_context(|| format!("Failed to read FMU at {}", fmu_path.display()))?;
    let major = min_desc
        .major_version()
        .context("Failed to determine FMI version")?;

    match format {
        InspectFormat::ModelDescription => match major {
            MajorVersion::FMI2 => inspect_model_description::<fmi::fmi2::import::Fmi2Import>(fmu_path)?,
            MajorVersion::FMI3 => inspect_model_description::<fmi::fmi3::import::Fmi3Import>(fmu_path)?,
            MajorVersion::FMI1 => anyhow::bail!("FMI 1.0 is not supported"),
        },
        InspectFormat::Debug => match major {
            MajorVersion::FMI2 => inspect_debug::<fmi::fmi2::import::Fmi2Import>(fmu_path, major)?,
            MajorVersion::FMI3 => inspect_debug::<fmi::fmi3::import::Fmi3Import>(fmu_path, major)?,
            MajorVersion::FMI1 => anyhow::bail!("FMI 1.0 is not supported"),
        },
    }

    Ok(())
}

fn inspect_model_description<Imp: FmiImport>(fmu_path: &Path) -> Result<()> {
    let import: Imp = fmi::import::from_path(fmu_path)
        .with_context(|| format!("Failed to import FMU at {}", fmu_path.display()))?;
    let xml = import
        .model_description()
        .serialize()
        .context("Failed to serialize model description")?;
    println!("{xml}");
    Ok(())
}

fn inspect_debug<Imp: FmiImport>(fmu_path: &Path, major: MajorVersion) -> Result<()> {
    let import: Imp = fmi::import::from_path(fmu_path)
        .with_context(|| format!("Failed to import FMU at {}", fmu_path.display()))?;
    let model_desc = import.model_description();

    println!("FMU: {}", fmu_path.display());
    println!(
        "Model: {} (FMI {})",
        model_desc.model_name(),
        model_desc.version_string()
    );
    println!("Entries:");
    let mut entries = list_archive_entries(import.archive_path())
        .context("Failed to list extracted FMU contents")?;
    entries.sort_by(|(a, _), (b, _)| a.cmp(b));
    for (path, size) in entries {
        println!("  - {} ({} bytes)", path.display(), size);
    }

    if major == MajorVersion::FMI3 {
        let build_path = import.archive_path().join("buildDescription.xml");
        if build_path.exists() {
            let build_xml = std::fs::read_to_string(&build_path)
                .with_context(|| format!("Failed to read {}", build_path.display()))?;
            match schema::deserialize::<fmi::fmi3::schema::Fmi3BuildDescription>(&build_xml) {
                Ok(build) => {
                    let pretty = schema::serialize(&build, true)?;
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
    }

    println!("\nModelDescription: use --format model-description to view XML.");
    Ok(())
}

fn list_archive_entries(root: &Path) -> Result<Vec<(PathBuf, u64)>> {
    let mut entries = Vec::new();
    collect_entries(root, root, &mut entries)?;
    Ok(entries)
}

fn collect_entries(root: &Path, dir: &Path, entries: &mut Vec<(PathBuf, u64)>) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_entries(root, &path, entries)?;
        } else {
            let metadata = entry.metadata()?;
            let rel = path.strip_prefix(root).unwrap_or(&path).to_path_buf();
            entries.push((rel, metadata.len()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn list_archive_entries_returns_files() {
        let root = tempdir().expect("temp dir");
        let root_path = root.path();
        std::fs::write(root_path.join("modelDescription.xml"), "test").expect("write file");
        std::fs::create_dir_all(root_path.join("nested")).expect("create nested");
        std::fs::write(
            root_path.join("nested").join("resources.bin"),
            "data",
        )
        .expect("write nested");

        let entries = list_archive_entries(root_path).expect("list entries");
        let names: Vec<_> = entries
            .into_iter()
            .map(|(path, _)| path.to_string_lossy().to_string())
            .collect();
        assert!(names.contains(&"modelDescription.xml".to_string()));
        assert!(names.contains(&format!(
            "nested{}resources.bin",
            std::path::MAIN_SEPARATOR
        )));
    }
}
