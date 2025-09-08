//! FMI 3.0 Build Description XML Schema
//!
//! This module provides Rust data structures for parsing and generating
//! FMI 3.0 Build Description XML files (`buildDescription.xml`).
//!
//! The Build Description XML file contains information about how to compile
//! the source code of an FMU, including:
//! - Source files and compilation settings
//! - Preprocessor definitions
//! - Include directories
//! - Library dependencies
//! - Platform-specific configurations
//!
//! # Example
//!
//! ```rust
//! use fmi_schema::fmi3::{Fmi3BuildDescription, BuildConfiguration, SourceFileSet, SourceFile};
//!
//! // Parse a build description from XML
//! let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
//! <fmiBuildDescription fmiVersion="3.0">
//!     <BuildConfiguration modelIdentifier="MyModel" platform="linux64">
//!         <SourceFileSet language="C" compiler="gcc">
//!             <SourceFile name="src/model.c"/>
//!         </SourceFileSet>
//!     </BuildConfiguration>
//! </fmiBuildDescription>"#;
//!
//! let build_desc: Fmi3BuildDescription = yaserde::de::from_str(xml).unwrap();
//! assert_eq!(build_desc.fmi_version, "3.0");
//! assert_eq!(build_desc.build_configurations.len(), 1);
//! ```

use yaserde_derive::{YaDeserialize, YaSerialize};

use super::Annotations;

/// Root element for FMI 3.0 build description XML
#[derive(Default, Debug, PartialEq, YaDeserialize, YaSerialize)]
#[yaserde(rename = "fmiBuildDescription")]
pub struct Fmi3BuildDescription {
    /// Version of FMI that was used to generate the XML file.
    /// Must match the pattern 3[.](0|[1-9][0-9]*)([.](0|[1-9][0-9]*))?(-.+)?
    #[yaserde(attribute = true, rename = "fmiVersion")]
    pub fmi_version: String,

    /// Build configurations for different platforms and settings
    #[yaserde(rename = "BuildConfiguration")]
    pub build_configurations: Vec<BuildConfiguration>,

    /// Optional annotations
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
}

/// Build configuration for a specific platform and model identifier
#[derive(Default, Debug, PartialEq, YaDeserialize, YaSerialize)]
#[yaserde(rename = "BuildConfiguration")]
pub struct BuildConfiguration {
    /// Model identifier that this build configuration applies to
    #[yaserde(attribute = true, rename = "modelIdentifier")]
    pub model_identifier: String,

    /// Target platform (e.g., "linux64", "win32", "darwin64")
    #[yaserde(attribute = true)]
    pub platform: Option<String>,

    /// Optional description of this build configuration
    #[yaserde(attribute = true)]
    pub description: Option<String>,

    /// Source file sets for compilation
    #[yaserde(rename = "SourceFileSet")]
    pub source_file_sets: Vec<SourceFileSet>,

    /// External libraries to link against
    #[yaserde(rename = "Library")]
    pub libraries: Vec<Library>,

    /// Optional annotations
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
}

/// Set of source files with compilation settings
#[derive(Default, Debug, PartialEq, YaDeserialize, YaSerialize)]
#[yaserde(rename = "SourceFileSet")]
pub struct SourceFileSet {
    /// Optional name for this source file set
    #[yaserde(attribute = true)]
    pub name: Option<String>,

    /// Programming language (e.g., "C", "C++", "Fortran")
    #[yaserde(attribute = true)]
    pub language: Option<String>,

    /// Compiler to use (e.g., "gcc", "clang", "msvc")
    #[yaserde(attribute = true)]
    pub compiler: Option<String>,

    /// Additional compiler options
    #[yaserde(attribute = true, rename = "compilerOptions")]
    pub compiler_options: Option<String>,

    /// List of source files in this set
    #[yaserde(rename = "SourceFile")]
    pub source_files: Vec<SourceFile>,

    /// Preprocessor definitions
    #[yaserde(rename = "PreprocessorDefinition")]
    pub preprocessor_definitions: Vec<PreprocessorDefinition>,

    /// Include directories
    #[yaserde(rename = "IncludeDirectory")]
    pub include_directories: Vec<IncludeDirectory>,

    /// Optional annotations
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
}

/// Individual source file
#[derive(Default, Debug, PartialEq, YaDeserialize, YaSerialize)]
#[yaserde(rename = "SourceFile")]
pub struct SourceFile {
    /// Name/path of the source file
    #[yaserde(attribute = true)]
    pub name: String,

    /// Optional annotations
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
}

/// Preprocessor definition/macro
#[derive(Default, Debug, PartialEq, YaDeserialize, YaSerialize)]
#[yaserde(rename = "PreprocessorDefinition")]
pub struct PreprocessorDefinition {
    /// Name of the preprocessor definition
    #[yaserde(attribute = true)]
    pub name: String,

    /// Whether this definition is optional
    #[yaserde(attribute = true)]
    pub optional: Option<bool>,

    /// Value of the preprocessor definition
    #[yaserde(attribute = true)]
    pub value: Option<String>,

    /// Description of this preprocessor definition
    #[yaserde(attribute = true)]
    pub description: Option<String>,

    /// Possible option values
    #[yaserde(rename = "Option")]
    pub options: Vec<PreprocessorOption>,

    /// Optional annotations
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
}

/// Option for a preprocessor definition
#[derive(Default, Debug, PartialEq, YaDeserialize, YaSerialize)]
#[yaserde(rename = "Option")]
pub struct PreprocessorOption {
    /// Value of the option
    #[yaserde(attribute = true)]
    pub value: Option<String>,

    /// Description of the option
    #[yaserde(attribute = true)]
    pub description: Option<String>,
}

/// Include directory for compilation
#[derive(Default, Debug, PartialEq, YaDeserialize, YaSerialize)]
#[yaserde(rename = "IncludeDirectory")]
pub struct IncludeDirectory {
    /// Name/path of the include directory
    #[yaserde(attribute = true)]
    pub name: String,

    /// Optional annotations
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
}

/// External library dependency
#[derive(Default, Debug, PartialEq, YaDeserialize, YaSerialize)]
#[yaserde(rename = "Library")]
pub struct Library {
    /// Name of the library
    #[yaserde(attribute = true)]
    pub name: String,

    /// Version of the library
    #[yaserde(attribute = true)]
    pub version: Option<String>,

    /// Whether this is an external library (not part of the FMU)
    #[yaserde(attribute = true)]
    pub external: Option<bool>,

    /// Description of the library
    #[yaserde(attribute = true)]
    pub description: Option<String>,

    /// Optional annotations
    #[yaserde(rename = "Annotations")]
    pub annotations: Option<Annotations>,
}
