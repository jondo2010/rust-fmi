//! # FMU Import Module
//!
//! This module provides functionality for importing and working with Functional Mockup Units (FMUs).
//! It handles the complete import process including:
//!
//! - Extracting FMU archives (ZIP files)
//! - Parsing the `modelDescription.xml` file
//! - Setting up the temporary directory structure
//! - Preparing for shared library loading
//!
//! ## Overview
//!
//! FMUs are distributed as ZIP archives containing the model description XML, shared libraries for
//! different platforms, and optional resources. This module provides both high-level and low-level
//! functions to work with these archives.
//!
//! ## Usage Patterns
//!
//! ### Standard Import Workflow
//!
//! ```rust,no_run
//! use fmi::{import, fmi3::import::Fmi3Import, traits::FmiImport};
//!
//! // Load an FMU from a file path
//! let import: Fmi3Import = import::from_path("path/to/model.fmu")?;
//! println!("Loaded FMI {} model: {}",
//!          import.model_description().fmi_version,
//!          import.model_description().model_name);
//! # Ok::<(), fmi::Error>(())
//! ```
//!
//! ### Version Detection Before Import
//!
//! ```rust,no_run
//! use fmi::{import, schema::{MajorVersion, traits::FmiModelDescription}};
//!
//! // Check the FMI version without full extraction
//! let model_desc = import::peek_descr_path("path/to/model.fmu")?;
//! match model_desc.major_version()? {
//!     MajorVersion::FMI2 => {
//!         let import: fmi::fmi2::import::Fmi2Import = import::from_path("path/to/model.fmu")?;
//!         // Work with FMI 2.0 import...
//!     }
//!     MajorVersion::FMI3 => {
//!         let import: fmi::fmi3::import::Fmi3Import = import::from_path("path/to/model.fmu")?;
//!         // Work with FMI 3.0 import...
//!     }
//!     _ => return Err(fmi::Error::UnsupportedFmiVersion(model_desc.major_version()?)),
//! }
//! # Ok::<(), fmi::Error>(())
//! ```
//!
//! ### Working with In-Memory FMU Data
//!
//! ```rust,no_run
//! use fmi::{import, fmi3::import::Fmi3Import};
//! use std::io::Cursor;
//!
//! let fmu_data: Vec<u8> = std::fs::read("path/to/model.fmu")?;
//! let cursor = Cursor::new(fmu_data);
//! let import: Fmi3Import = import::new(cursor)?;
//! # Ok::<(), fmi::Error>(())
//! ```
//!
//! ## Error Handling
//!
//! All functions in this module return [`Result`] types with detailed error information.
//! Common error scenarios include:
//!
//! - Invalid or corrupted FMU archives
//! - Missing or malformed `modelDescription.xml`
//! - I/O errors during extraction
//! - Unsupported FMI versions
//!
//! See [`crate::Error`] for the complete list of error types.
//!
//! ## Related Modules
//!
//! - [`crate::traits::FmiImport`] - Core import trait implemented by version-specific imports
#![cfg_attr(
    feature = "fmi2",
    doc = "- [`crate::fmi2::import`] - FMI 2.0 specific import functionality"
)]
#![cfg_attr(
    not(feature = "fmi2"),
    doc = "- FMI 2.0 specific import functionality (enable the `fmi2` feature)"
)]
#![cfg_attr(
    feature = "fmi3",
    doc = "- [`crate::fmi3::import`] - FMI 3.0 specific import functionality"
)]
#![cfg_attr(
    not(feature = "fmi3"),
    doc = "- FMI 3.0 specific import functionality (enable the `fmi3` feature)"
)]

use std::{
    io::{Read, Seek},
    path::Path,
};

use crate::{Error, traits::FmiImport};

use fmi_schema::{minimal::MinModelDescription as MinModel, traits::FmiModelDescription};

/// Standard filename for the FMU model description file within the archive.
///
/// According to the FMI standard, every FMU must contain a file named
/// `modelDescription.xml` at the root level of the ZIP archive. This file
/// contains the complete model metadata including variables, capabilities,
/// and platform-specific information.
const MODEL_DESCRIPTION: &str = "modelDescription.xml";

/// Quickly inspect an FMU's model description without full extraction.
///
/// This function opens an FMU file and reads only the `modelDescription.xml` file
/// to extract basic metadata such as the FMI version and model name. This is useful
/// for version detection and validation before committing to a full import.
///
/// # Arguments
///
/// * `path` - Path to the FMU file (`.fmu` extension expected)
///
/// # Returns
///
/// Returns a [`fmi_schema::minimal::MinModelDescription`] containing
/// the essential model metadata including:
/// - FMI version
/// - Model name
/// - Model identifier
/// - Basic model attributes
///
/// # Errors
///
/// This function will return an error if:
/// - The file cannot be opened or read
/// - The file is not a valid ZIP archive
/// - The `modelDescription.xml` file is missing or corrupted
/// - The XML cannot be parsed
///
/// # Examples
///
/// ```rust,no_run
/// use fmi::import;
///
/// let model_desc = import::peek_descr_path("path/to/model.fmu")?;
/// println!("Model: {} (FMI {})",
///          model_desc.model_name,
///          model_desc.fmi_version);
/// # Ok::<(), fmi::Error>(())
/// ```
///
/// # See Also
///
/// - [`peek_descr`] for working with in-memory data
/// - [`from_path`] for full FMU import
pub fn peek_descr_path(path: impl AsRef<Path>) -> Result<MinModel, Error> {
    let file = std::fs::File::open(path.as_ref())?;
    peek_descr(file)
}

/// Quickly inspect an FMU's model description from a reader without full extraction.
///
/// This function reads only the `modelDescription.xml` file from an FMU archive
/// provided as a reader to extract basic metadata. This is useful when working
/// with FMU data from memory, network streams, or other non-file sources.
///
/// # Arguments
///
/// * `reader` - Any reader that implements [`Read`] + [`Seek`] containing FMU data
///
/// # Type Parameters
///
/// * `R` - The reader type, must implement [`Read`] + [`Seek`] for ZIP archive access
///
/// # Returns
///
/// Returns a [`fmi_schema::minimal::MinModelDescription`] containing
/// the essential model metadata.
///
/// # Errors
///
/// This function will return an error if:
/// - The reader data is not a valid ZIP archive
/// - The `modelDescription.xml` file is missing from the archive
/// - The XML content cannot be parsed as valid FMI model description
///
/// # Examples
///
/// ```rust,no_run
/// use fmi::import;
/// use std::io::Cursor;
///
/// let fmu_data: Vec<u8> = std::fs::read("path/to/model.fmu")?;
/// let cursor = Cursor::new(fmu_data);
/// let model_desc = import::peek_descr(cursor)?;
/// println!("Found FMI {} model: {}",
///          model_desc.fmi_version,
///          model_desc.model_name);
/// # Ok::<(), fmi::Error>(())
/// ```
///
/// # See Also
///
/// - [`peek_descr_path`] for file-based inspection
/// - [`new`] for full import from reader
pub fn peek_descr<R: Read + Seek>(reader: R) -> Result<MinModel, Error> {
    let mut archive = zip::ZipArchive::new(reader)?;
    let mut descr_file = archive
        .by_name(MODEL_DESCRIPTION)
        .map_err(|e| Error::ArchiveStructure(e.to_string()))?;
    let mut descr_xml = String::new();
    descr_file.read_to_string(&mut descr_xml)?;
    let descr = MinModel::deserialize(&descr_xml)?;
    log::debug!(
        "Found FMI {} named '{}'",
        descr.fmi_version,
        descr.model_name
    );
    Ok(descr)
}

/// Import an FMU from a file path with full extraction and setup.
///
/// This function performs a complete FMU import process:
/// 1. Opens and validates the FMU file
/// 2. Extracts the entire archive to a temporary directory
/// 3. Parses the `modelDescription.xml` file
/// 4. Creates the appropriate import instance based on the FMI version
///
/// The resulting import object can be used to instantiate FMU instances for
/// simulation or model exchange.
///
/// # Arguments
///
/// * `path` - Path to the FMU file (typically with `.fmu` extension)
///
/// # Type Parameters
///
/// * `Imp` - The import type that implements [`FmiImport`], determines FMI version
///
/// # Returns
///
/// Returns an import instance of the specified type that provides access to:
/// - The parsed model description
/// - The extracted FMU directory
/// - Functions to instantiate FMU instances
/// - Methods to load the shared library
///
/// # Errors
///
/// This function will return an error if:
/// - The file cannot be opened or is not found
/// - The file is not a valid FMU (ZIP) archive
/// - Extraction to temporary directory fails
/// - The `modelDescription.xml` is missing or invalid
/// - The FMI version doesn't match the expected import type
///
/// # Examples
///
/// ```rust,no_run
/// use fmi::{import, fmi3::import::Fmi3Import, traits::FmiImport, fmi3::Fmi3Model};
///
/// // Import an FMI 3.0 FMU
/// let import: Fmi3Import = import::from_path("path/to/model.fmu")?;
///
/// // Access model information
/// let model_desc = import.model_description();
/// println!("Loaded model: {} ({})",
///          model_desc.model_name,
///          model_desc.fmi_version);
///
/// // Create instances (if supported)
/// if model_desc.model_exchange.is_some() {
///     let me_instance = import.instantiate_me("instance1", false, true)?;
/// }
/// # Ok::<(), fmi::Error>(())
/// ```
///
/// # Performance Notes
///
/// This function extracts the entire FMU to a temporary directory, which may be
/// expensive for large FMUs. Consider using [`peek_descr_path`] first for version
/// detection if you need to handle multiple FMI versions.
///
/// # See Also
///
/// - [`new`] for importing from a reader
/// - [`peek_descr_path`] for lightweight version detection
/// - [`crate::traits::FmiImport`] for the import trait interface
pub fn from_path<Imp: FmiImport>(path: impl AsRef<Path>) -> Result<Imp, Error> {
    let file = std::fs::File::open(path.as_ref())?;
    log::debug!("Opening FMU file {:?}", path.as_ref());
    new(file)
}

/// Import an FMU from a reader with full extraction and setup.
///
/// This function performs a complete FMU import process from any reader containing
/// FMU data. It handles the same extraction and parsing workflow as [`from_path`]
/// but works with in-memory data, network streams, or other reader sources.
///
/// # Arguments
///
/// * `reader` - Any reader containing FMU archive data
///
/// # Type Parameters
///
/// * `R` - The reader type, must implement [`Read`] + [`Seek`] for ZIP access
/// * `Imp` - The import type implementing [`FmiImport`], determines expected FMI version
///
/// # Returns
///
/// Returns an import instance providing full access to the FMU's capabilities
/// including model description, instance creation, and shared library loading.
///
/// # Errors
///
/// This function will return an error if:
/// - The reader data is not a valid FMU (ZIP) archive
/// - Archive extraction fails due to I/O issues or insufficient disk space
/// - The `modelDescription.xml` is missing, corrupted, or doesn't match expected version
/// - Temporary directory creation fails
///
/// # Examples
///
/// ```rust,no_run
/// use fmi::{import, fmi2::import::Fmi2Import, traits::FmiImport};
/// use std::io::Cursor;
///
/// // Load FMU data into memory
/// let fmu_bytes = std::fs::read("path/to/model.fmu")?;
/// let cursor = Cursor::new(fmu_bytes);
///
/// // Import from memory
/// let import: Fmi2Import = import::new(cursor)?;
///
/// // Use the import normally
/// let model_desc = import.model_description();
/// println!("Imported {} from memory", model_desc.model_name);
/// # Ok::<(), fmi::Error>(())
/// ```
///
/// # Implementation Details
///
/// The function creates a temporary directory using [`tempfile::Builder`] with the
/// prefix "fmi-rs". This directory is automatically cleaned up when the import
/// instance is dropped. The entire FMU archive is extracted to this directory
/// to enable shared library loading and resource access.
///
/// # See Also
///
/// - [`from_path`] for file-based import
/// - [`peek_descr`] for lightweight metadata extraction
/// - [`crate::traits::FmiImport::new`] for the underlying trait method
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
