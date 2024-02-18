//! Utilities for fetching test data from Modelica's Reference-FMUs repository

use fetch_data::{ctor, FetchData};
use std::{
    fs::File,
    io::{Cursor, Read},
    path::PathBuf,
};

use fmi::Import;

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

impl ReferenceFmus {
    /// Fetch the released Modelica Reference-FMUs file
    pub fn new() -> anyhow::Result<Self> {
        let path = STATIC_FETCH_DATA.fetch_file(REF_ARCHIVE)?;
        let f = std::fs::File::open(&path)?;
        let archive = zip::ZipArchive::new(f)?;
        Ok(Self { archive })
    }

    /// Get a reference FMU from the archive
    pub fn get_reference_fmu(&mut self, name: &str, version: &str) -> anyhow::Result<Import> {
        let mut f = self.archive.by_name(&format!("{}/{}.fmu", version, name))?;
        // Read f into a Vec<u8> that can be used to create a new Import
        let mut buf = Vec::new();
        f.read_to_end(buf.as_mut())?;
        Ok(Import::new(Cursor::new(buf))?)
    }

    /// Extract a reference FMU from the reference archive, relative to the zip file, and returns the path
    pub fn extract_reference_fmu(&mut self, name: &str, version: &str) -> anyhow::Result<PathBuf> {
        let filename = format!("{}/{}.fmu", version, name);
        let mut f = self.archive.by_name(&filename)?;
        let mut buf = Vec::new();
        f.read_to_end(buf.as_mut())?;
        let path = STATIC_FETCH_DATA.cache_dir().unwrap().join(&filename);
        std::fs::write(&path, buf)?;
        Ok(path)
    }
}

#[test]
fn test_reference_fmus() {
    use fmi::FmiImport;
    let mut reference_fmus = ReferenceFmus::new().unwrap();
    let fmu = reference_fmus
        .get_reference_fmu("BouncingBall", "2.0")
        .unwrap();
    println!(
        "{:?}",
        fmu.as_fmi2().unwrap().model_description().model_name
    );
}

#[cfg(feature = "disabled")]
#[test]
fn print_registry_contents() {
    let registry_contents = STATIC_FETCH_DATA
        .gen_registry_contents([REF_ARCHIVE])
        .unwrap();
    println!("{registry_contents}");
}
