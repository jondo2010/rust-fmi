#![doc=include_str!( "../README.md")]
#![deny(unsafe_code)]
#![deny(clippy::all)]

use anyhow::Context;
use fetch_data::{ctor, FetchData};
use fmi::{schema::MajorVersion, traits::FmiImport};
use std::{
    fs::File,
    io::{Cursor, Read},
};
use tempfile::NamedTempFile;

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

impl std::fmt::Debug for ReferenceFmus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReferenceFmus")
            .field("archive", &self.archive.comment())
            .finish()
    }
}

impl ReferenceFmus {
    /// Fetch the released Modelica Reference-FMUs file
    pub fn new() -> anyhow::Result<Self> {
        let path = STATIC_FETCH_DATA
            .fetch_file(REF_ARCHIVE)
            .context(format!("Fetch {REF_ARCHIVE}"))?;
        let f = std::fs::File::open(&path).context(format!("Open {path:?}"))?;
        let archive = zip::ZipArchive::new(f)?;
        Ok(Self { archive })
    }

    pub fn get_reference_fmu<Imp: FmiImport>(&mut self, name: &str) -> anyhow::Result<Imp> {
        let version = Imp::MAJOR_VERSION.to_string();
        let mut f = self.archive.by_name(&format!("{version}/{name}.fmu"))?;
        // Read f into a Vec<u8> that can be used to create a new Import
        let mut buf = Vec::new();
        f.read_to_end(buf.as_mut())?;
        Ok(fmi::import::new(Cursor::new(buf))?)
    }

    /// Extract a reference FMU from the reference archive into a temporary file
    pub fn extract_reference_fmu(
        &mut self,
        name: &str,
        version: MajorVersion,
    ) -> anyhow::Result<NamedTempFile> {
        let version = version.to_string();
        let filename = format!("{version}/{name}.fmu");
        let mut fin = self.archive.by_name(&filename).context("Open {filename}")?;
        let mut fout = tempfile::NamedTempFile::new()?;
        std::io::copy(fin.by_ref(), fout.as_file_mut())
            .context("Extracting {path:?} to tempfile")?;
        Ok(fout)
    }
}

#[test]
fn test_reference_fmus() {
    use fmi::traits::FmiImport;
    let mut reference_fmus = ReferenceFmus::new().unwrap();
    let fmu: fmi::fmi2::import::Fmi2Import =
        reference_fmus.get_reference_fmu("BouncingBall").unwrap();
    assert_eq!(fmu.model_description().fmi_version, "2.0");
    assert_eq!(fmu.model_description().model_name, "BouncingBall");
    let fmu: fmi::fmi3::import::Fmi3Import =
        reference_fmus.get_reference_fmu("BouncingBall").unwrap();
    assert_eq!(fmu.model_description().fmi_version, "3.0");
    assert_eq!(fmu.model_description().model_name, "BouncingBall");
}

#[cfg(false)]
#[test]
fn print_registry_contents() {
    let registry_contents = STATIC_FETCH_DATA
        .gen_registry_contents([REF_ARCHIVE])
        .unwrap();
    println!("{registry_contents}");
}
