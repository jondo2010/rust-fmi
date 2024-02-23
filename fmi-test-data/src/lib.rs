//! Utilities for fetching test data from Modelica's Reference-FMUs repository

use anyhow::Context;
use fetch_data::{ctor, FetchData};
use fmi::{fmi2::import::Fmi2Import, fmi3::import::Fmi3Import};
use std::{
    fs::File,
    io::{Cursor, Read},
    path::PathBuf,
};

const REF_ARCHIVE: &str = "Reference-FMUs-0.0.29.zip";
const REF_URL: &str = "https://github.com/modelica/Reference-FMUs/releases/download/v0.0.29/";

#[ctor]
static STATIC_FETCH_DATA: FetchData = FetchData::new(
    include_str!("registry.txt"),
    REF_URL,
    "FMU_DATA_DIR",
    "org",
    "modelica",
    "reference-fmus",
);

/// A Rust interface to the Modelica Reference-FMUs, downloaded as an archive using `fetch_data`
pub struct ReferenceFmus {
    archive: zip::ZipArchive<File>,
}

fn version_str(version: usize) -> anyhow::Result<&'static str> {
    match version {
        1 => anyhow::bail!("Version 1.0 is not supported"),
        2 => Ok("2.0"),
        3 => Ok("3.0"),
        _ => anyhow::bail!("Invalid version"),
    }
}

impl ReferenceFmus {
    /// Fetch the released Modelica Reference-FMUs file
    pub fn new() -> anyhow::Result<Self> {
        let path = STATIC_FETCH_DATA
            .fetch_file(REF_ARCHIVE)
            .context(format!("Fetch {REF_ARCHIVE}"))?;
        let f = std::fs::File::open(&path).context(format!("Open {:?}", path))?;
        let archive = zip::ZipArchive::new(f)?;
        Ok(Self { archive })
    }

    /// Get a 2.0 reference FMU from the archive
    pub fn get_reference_fmu_fmi2(&mut self, name: &str) -> anyhow::Result<Fmi2Import> {
        let mut f = self.archive.by_name(&format!("{}/{}.fmu", "2.0", name))?;
        // Read f into a Vec<u8> that can be used to create a new Import
        let mut buf = Vec::new();
        f.read_to_end(buf.as_mut())?;
        Ok(fmi::import::new(Cursor::new(buf))?)
    }

    /// Get a 3.0 reference FMU from the archive
    pub fn get_reference_fmu_fmi3(&mut self, name: &str) -> anyhow::Result<Fmi3Import> {
        let mut f = self.archive.by_name(&format!("{}/{}.fmu", "3.0", name))?;
        // Read f into a Vec<u8> that can be used to create a new Import
        let mut buf = Vec::new();
        f.read_to_end(buf.as_mut())?;
        Ok(fmi::import::new(Cursor::new(buf))?)
    }

    /// Extract a reference FMU from the reference archive, relative to the zip file, and returns the path
    pub fn extract_reference_fmu(&mut self, name: &str, version: usize) -> anyhow::Result<PathBuf> {
        let v = version_str(version)?;
        let filename = format!("{v}/{name}.fmu");
        let mut f = self.archive.by_name(&filename).context("Open {filename}")?;
        let mut buf = Vec::new();
        f.read_to_end(buf.as_mut())?;
        let path = STATIC_FETCH_DATA.cache_dir().unwrap().join(&filename);
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, buf).context(format!("Extracting {path:?}"))?;
        Ok(path)
    }
}

#[test]
fn test_reference_fmus() {
    use fmi::traits::FmiImport;
    let mut reference_fmus = ReferenceFmus::new().unwrap();
    let fmu = reference_fmus
        .get_reference_fmu_fmi2("BouncingBall")
        .unwrap();
    assert_eq!(fmu.model_description().model_name, "BouncingBall");
}

#[cfg(feature = "disabled")]
#[test]
fn print_registry_contents() {
    let registry_contents = STATIC_FETCH_DATA
        .gen_registry_contents([REF_ARCHIVE])
        .unwrap();
    println!("{registry_contents}");
}
