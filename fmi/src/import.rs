//!  Import is responsible for extracting the FMU, parsing the modelDescription XML and loading the shared library.

use std::{
    io::{Read, Seek},
    path::Path,
    str::FromStr,
};

use crate::{traits::FmiImport, Error};

use fmi_schema::minimal::ModelDescription as MinModel;

const MODEL_DESCRIPTION: &str = "modelDescription.xml";

/// Peek at the modelDescription XML without extracting the FMU
pub fn peek_descr_path(path: impl AsRef<Path>) -> Result<MinModel, Error> {
    let file = std::fs::File::open(path.as_ref())?;
    peek_descr(file)
}

/// Peek at the modelDescription XML without extracting the FMU
pub fn peek_descr<R: Read + Seek>(reader: R) -> Result<MinModel, Error> {
    let mut archive = zip::ZipArchive::new(reader)?;
    let mut descr_file = archive
        .by_name(MODEL_DESCRIPTION)
        .map_err(|e| Error::ArchiveStructure(e.to_string()))?;
    let mut descr_xml = String::new();
    descr_file.read_to_string(&mut descr_xml)?;
    let descr = MinModel::from_str(&descr_xml)?;
    log::debug!(
        "Found FMI {} named '{}'",
        descr.fmi_version,
        descr.model_name
    );
    Ok(descr)
}

/// Creates a new Import by extracting the FMU and parsing the modelDescription XML
pub fn from_path<Imp: FmiImport>(path: impl AsRef<Path>) -> Result<Imp, Error> {
    let file = std::fs::File::open(path.as_ref())?;
    log::debug!("Opening FMU file {:?}", path.as_ref());
    new(file)
}

/// Creates a new Import by extracting the FMU and parsing the modelDescription XML
pub fn new<R: Read + Seek, Imp: FmiImport>(reader: R) -> Result<Imp, Error> {
    let mut archive = zip::ZipArchive::new(reader)?;
    let temp_dir = tempfile::Builder::new().prefix("fmi-rs").tempdir()?;
    log::debug!("Extracting into {temp_dir:?}");
    archive.extract(&temp_dir)?;

    // Open and read the modelDescription XML into a string
    let descr_file_path = temp_dir.path().join(MODEL_DESCRIPTION);
    let descr_xml = std::fs::read_to_string(descr_file_path)?;

    Imp::new(temp_dir, &descr_xml)
}
