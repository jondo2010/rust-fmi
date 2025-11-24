//! Test FMI 3.0 build description schema by parsing the FMI3BuildDescription.xml file.

#[test]
#[cfg(feature = "fmi3")]
fn test_fmi3_build_description() {
    use fmi_schema::fmi3::Fmi3BuildDescription;

    let test_file = std::env::current_dir()
        .map(|path| path.join("tests/FMI3BuildDescription.xml"))
        .unwrap();
    let xml_content = std::fs::read_to_string(test_file).unwrap();
    let build_desc: Fmi3BuildDescription = fmi_schema::deserialize(&xml_content).unwrap();

    // Verify basic structure
    assert_eq!(build_desc.fmi_version, "3.0");
    assert_eq!(build_desc.build_configurations.len(), 2);

    // Test first build configuration (Linux)
    let linux_config = &build_desc.build_configurations[0];
    assert_eq!(linux_config.model_identifier, "BouncingBall");
    assert_eq!(linux_config.platform.as_deref(), Some("linux64"));
    assert_eq!(
        linux_config.description.as_deref(),
        Some("Linux 64-bit build")
    );

    // Test source file set
    assert_eq!(linux_config.source_file_sets.len(), 1);
    let source_set = &linux_config.source_file_sets[0];
    assert_eq!(source_set.name.as_deref(), Some("main"));
    assert_eq!(source_set.language.as_deref(), Some("C"));
    assert_eq!(source_set.compiler.as_deref(), Some("gcc"));
    assert_eq!(source_set.compiler_options.as_deref(), Some("-O2 -Wall"));

    // Test source files
    assert_eq!(source_set.source_files.len(), 2);
    assert_eq!(source_set.source_files[0].name, "src/bouncing_ball.c");
    assert_eq!(source_set.source_files[1].name, "src/fmi_functions.c");

    // Test preprocessor definitions
    assert_eq!(source_set.preprocessor_definitions.len(), 2);
    let fmi_version_def = &source_set.preprocessor_definitions[0];
    assert_eq!(fmi_version_def.name, "FMI_VERSION");
    assert_eq!(fmi_version_def.value.as_deref(), Some("3"));
    assert_eq!(fmi_version_def.description.as_deref(), Some("FMI version"));

    let debug_def = &source_set.preprocessor_definitions[1];
    assert_eq!(debug_def.name, "DEBUG");
    assert_eq!(debug_def.optional, Some(true));
    assert_eq!(debug_def.options.len(), 2);
    assert_eq!(debug_def.options[0].value.as_deref(), Some("0"));
    assert_eq!(
        debug_def.options[0].description.as_deref(),
        Some("Disable debug output")
    );

    // Test include directories
    assert_eq!(source_set.include_directories.len(), 2);
    assert_eq!(source_set.include_directories[0].name, "include");
    assert_eq!(source_set.include_directories[1].name, "fmi/include");

    // Test libraries
    assert_eq!(linux_config.libraries.len(), 2);
    assert_eq!(linux_config.libraries[0].name, "m");
    assert_eq!(
        linux_config.libraries[0].description.as_deref(),
        Some("Math library")
    );
    assert_eq!(linux_config.libraries[1].name, "dl");
    assert_eq!(linux_config.libraries[1].external, Some(true));

    // Test second build configuration (Windows)
    let windows_config = &build_desc.build_configurations[1];
    assert_eq!(windows_config.model_identifier, "BouncingBall");
    assert_eq!(windows_config.platform.as_deref(), Some("win64"));
    assert_eq!(
        windows_config.description.as_deref(),
        Some("Windows 64-bit build")
    );

    let win_source_set = &windows_config.source_file_sets[0];
    assert_eq!(win_source_set.compiler.as_deref(), Some("msvc"));
    assert_eq!(win_source_set.compiler_options.as_deref(), Some("/O2 /W3"));

    // Verify Windows-specific preprocessor definitions
    let win_preprocessor_defs = &win_source_set.preprocessor_definitions;
    assert_eq!(win_preprocessor_defs.len(), 2);
    assert_eq!(win_preprocessor_defs[1].name, "WIN32");
    assert_eq!(win_preprocessor_defs[1].value.as_deref(), Some("1"));
}

#[test]
#[cfg(feature = "fmi3")]
fn test_fmi3_build_description_serialization() {
    use fmi_schema::fmi3::{
        BuildConfiguration, Fmi3BuildDescription, IncludeDirectory, Library,
        PreprocessorDefinition, SourceFile, SourceFileSet,
    };

    // Create a simple build description programmatically
    let build_desc = Fmi3BuildDescription {
        fmi_version: "3.0".to_string(),
        build_configurations: vec![BuildConfiguration {
            model_identifier: "TestModel".to_string(),
            platform: Some("linux64".to_string()),
            description: Some("Test build configuration".to_string()),
            source_file_sets: vec![SourceFileSet {
                name: Some("core".to_string()),
                language: Some("C".to_string()),
                compiler: Some("gcc".to_string()),
                source_files: vec![SourceFile {
                    name: "test.c".to_string(),
                    annotations: None,
                }],
                preprocessor_definitions: vec![PreprocessorDefinition {
                    name: "TEST_DEFINE".to_string(),
                    value: Some("1".to_string()),
                    ..Default::default()
                }],
                include_directories: vec![IncludeDirectory {
                    name: "include".to_string(),
                    annotations: None,
                }],
                ..Default::default()
            }],
            libraries: vec![Library {
                name: "m".to_string(),
                ..Default::default()
            }],
            annotations: None,
        }],
        annotations: None,
    };

    // Test serialization to XML
    let xml = fmi_schema::serialize(&build_desc, false).unwrap();
    assert!(xml.contains(r#"fmiVersion="3.0""#));
    assert!(xml.contains(r#"modelIdentifier="TestModel""#));
    assert!(xml.contains(r#"platform="linux64""#));
    assert!(xml.contains("<SourceFile"));
    assert!(xml.contains(r#"name="test.c""#));

    // Test round-trip: serialize then deserialize
    let deserialized: Fmi3BuildDescription = fmi_schema::deserialize(&xml).unwrap();
    assert_eq!(deserialized.fmi_version, build_desc.fmi_version);
    assert_eq!(
        deserialized.build_configurations.len(),
        build_desc.build_configurations.len()
    );
    assert_eq!(
        deserialized.build_configurations[0].model_identifier,
        build_desc.build_configurations[0].model_identifier
    );
}
