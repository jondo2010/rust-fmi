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
//! let build_desc: Fmi3BuildDescription = fmi_schema::deserialize(xml).unwrap();
//! assert_eq!(build_desc.fmi_version, "3.0");
//! assert_eq!(build_desc.build_configurations.len(), 1);
//! ```

use super::Annotations;

/// Root element for FMI 3.0 build description XML
#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(
    tag = "fmiBuildDescription",
    strict(unknown_attribute, unknown_element)
)]
pub struct Fmi3BuildDescription {
    /// Version of FMI that was used to generate the XML file.
    /// Must match the pattern 3[.](0|[1-9][0-9]*)([.](0|[1-9][0-9]*))?(-.+)?
    #[xml(attr = "fmiVersion")]
    pub fmi_version: String,

    /// Build configurations for different platforms and settings
    #[xml(child = "BuildConfiguration")]
    pub build_configurations: Vec<BuildConfiguration>,

    /// Optional annotations
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
}

/// Build configuration for a specific platform and model identifier
#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "BuildConfiguration", strict(unknown_attribute, unknown_element))]
pub struct BuildConfiguration {
    /// Model identifier that this build configuration applies to
    #[xml(attr = "modelIdentifier")]
    pub model_identifier: String,

    /// Target platform (e.g., "linux64", "win32", "darwin64")
    #[xml(attr = "platform")]
    pub platform: Option<String>,

    /// Optional description of this build configuration
    #[xml(attr = "description")]
    pub description: Option<String>,

    /// Source file sets for compilation
    #[xml(child = "SourceFileSet")]
    pub source_file_sets: Vec<SourceFileSet>,

    /// External libraries to link against
    #[xml(child = "Library")]
    pub libraries: Vec<Library>,

    /// Optional annotations
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
}

/// Set of source files with compilation settings
#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "SourceFileSet", strict(unknown_attribute, unknown_element))]
pub struct SourceFileSet {
    /// Optional name for this source file set
    #[xml(attr = "name")]
    pub name: Option<String>,

    /// Programming language (e.g., "C", "C++", "Fortran")
    #[xml(attr = "language")]
    pub language: Option<String>,

    /// Compiler to use (e.g., "gcc", "clang", "msvc")
    #[xml(attr = "compiler")]
    pub compiler: Option<String>,

    /// Additional compiler options
    #[xml(attr = "compilerOptions")]
    pub compiler_options: Option<String>,

    /// List of source files in this set
    #[xml(child = "SourceFile")]
    pub source_files: Vec<SourceFile>,

    /// Preprocessor definitions
    #[xml(child = "PreprocessorDefinition")]
    pub preprocessor_definitions: Vec<PreprocessorDefinition>,

    /// Include directories
    #[xml(child = "IncludeDirectory")]
    pub include_directories: Vec<IncludeDirectory>,

    /// Optional annotations
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
}

/// Individual source file
#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "SourceFile", strict(unknown_attribute, unknown_element))]
pub struct SourceFile {
    /// Name/path of the source file
    #[xml(attr = "name")]
    pub name: String,

    /// Optional annotations
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
}

/// Preprocessor definition/macro
#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(
    tag = "PreprocessorDefinition",
    strict(unknown_attribute, unknown_element)
)]
pub struct PreprocessorDefinition {
    /// Name of the preprocessor definition
    #[xml(attr = "name")]
    pub name: String,

    /// Whether this definition is optional
    #[xml(attr = "optional")]
    pub optional: Option<bool>,

    /// Value of the preprocessor definition
    #[xml(attr = "value")]
    pub value: Option<String>,

    /// Description of this preprocessor definition
    #[xml(attr = "description")]
    pub description: Option<String>,

    /// Possible option values
    #[xml(child = "Option")]
    pub options: Vec<PreprocessorOption>,

    /// Optional annotations
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
}

/// Option for a preprocessor definition
#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Option", strict(unknown_attribute, unknown_element))]
pub struct PreprocessorOption {
    /// Value of the option
    #[xml(attr = "value")]
    pub value: Option<String>,

    /// Description of the option
    #[xml(attr = "description")]
    pub description: Option<String>,
}

/// Include directory for compilation
#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "IncludeDirectory", strict(unknown_attribute, unknown_element))]
pub struct IncludeDirectory {
    /// Name/path of the include directory
    #[xml(attr = "name")]
    pub name: String,

    /// Optional annotations
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
}

/// External library dependency
#[derive(Default, Debug, PartialEq, hard_xml::XmlRead, hard_xml::XmlWrite)]
#[xml(tag = "Library", strict(unknown_attribute, unknown_element))]
pub struct Library {
    /// Name of the library
    #[xml(attr = "name")]
    pub name: String,

    /// Version of the library
    #[xml(attr = "version")]
    pub version: Option<String>,

    /// Whether this is an external library (not part of the FMU)
    #[xml(attr = "external")]
    pub external: Option<bool>,

    /// Description of the library
    #[xml(attr = "description")]
    pub description: Option<String>,

    /// Optional annotations
    #[xml(child = "Annotations")]
    pub annotations: Option<Annotations>,
}

#[cfg(test)]
mod tests {
    use hard_xml::{XmlRead, XmlWrite};

    use super::*;

    #[test]
    fn defaults_are_empty_and_optional() {
        let description = Fmi3BuildDescription::default();
        let configuration = BuildConfiguration::default();
        let source_set = SourceFileSet::default();
        let source_file = SourceFile::default();
        let definition = PreprocessorDefinition::default();
        let option = PreprocessorOption::default();
        let include = IncludeDirectory::default();
        let library = Library::default();

        assert!(description.fmi_version.is_empty());
        assert!(description.build_configurations.is_empty());
        assert!(description.annotations.is_none());
        assert!(configuration.model_identifier.is_empty());
        assert!(configuration.platform.is_none());
        assert!(configuration.source_file_sets.is_empty());
        assert!(configuration.libraries.is_empty());
        assert!(source_set.name.is_none());
        assert!(source_set.source_files.is_empty());
        assert!(source_set.preprocessor_definitions.is_empty());
        assert!(source_set.include_directories.is_empty());
        assert!(source_file.name.is_empty());
        assert!(definition.name.is_empty());
        assert!(definition.options.is_empty());
        assert!(option.value.is_none());
        assert!(include.name.is_empty());
        assert!(library.name.is_empty());
        assert!(library.external.is_none());
    }

    #[test]
    fn full_build_description_round_trips_all_metadata() {
        let xml = r#"<fmiBuildDescription fmiVersion="3.0">
            <BuildConfiguration modelIdentifier="motor" platform="linux64" description="release">
                <SourceFileSet name="core" language="C" compiler="clang" compilerOptions="-O3">
                    <SourceFile name="src/motor.c"/>
                    <PreprocessorDefinition name="MODE" optional="true" value="fast" description="build mode">
                        <Option value="fast" description="optimized"/>
                    </PreprocessorDefinition>
                    <IncludeDirectory name="include"/>
                </SourceFileSet>
                <Library name="m" version="1.0" external="true" description="math"/>
            </BuildConfiguration>
        </fmiBuildDescription>"#;

        let parsed = Fmi3BuildDescription::from_str(xml).unwrap();
        let serialized = parsed.to_string().unwrap();
        let reparsed = Fmi3BuildDescription::from_str(&serialized).unwrap();

        assert_eq!(reparsed, parsed);
        let definition =
            &reparsed.build_configurations[0].source_file_sets[0].preprocessor_definitions[0];
        assert_eq!(definition.optional, Some(true));
        assert_eq!(
            definition.options[0].description.as_deref(),
            Some("optimized")
        );
        let library = &reparsed.build_configurations[0].libraries[0];
        assert_eq!(library.version.as_deref(), Some("1.0"));
        assert_eq!(library.external, Some(true));
    }

    #[test]
    fn strict_schema_rejects_unknown_attributes_and_children() {
        let unknown_attribute = r#"<fmiBuildDescription fmiVersion="3.0" unsupported="true"/>"#;
        let unknown_child =
            r#"<fmiBuildDescription fmiVersion="3.0"><Unsupported/></fmiBuildDescription>"#;

        assert!(Fmi3BuildDescription::from_str(unknown_attribute).is_err());
        assert!(Fmi3BuildDescription::from_str(unknown_child).is_err());
    }
}
