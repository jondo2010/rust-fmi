//! Tests for the `fmi_schema::serialize` and `fmi_schema::deserialize` functions.

#[cfg(feature = "fmi2")]
use fmi_schema::fmi2::{BaseUnit, DefaultExperiment, Fmi2ModelDescription, Fmi2Unit};
#[cfg(feature = "fmi3")]
use fmi_schema::fmi3::Fmi3ModelDescription;
use fmi_schema::traits::{
    DefaultExperiment as DefaultExperimentTrait, FmiInterfaceType, FmiModelDescription,
};
use fmi_schema::{deserialize, serialize};

#[test]
#[cfg(feature = "fmi2")]
fn test_serialize_deserialize_fmi2_model_description() {
    // Read the FMI2 test file
    let test_file = std::env::current_dir()
        .map(|path| path.join("tests/FMI2.xml"))
        .unwrap();
    let xml_content = std::fs::read_to_string(test_file).unwrap();

    // Deserialize from XML
    let original_md: Fmi2ModelDescription = deserialize(&xml_content).unwrap();

    // Serialize back to XML
    let serialized_xml = serialize(&original_md, false).unwrap();

    // Deserialize again from the serialized XML
    let round_trip_md: Fmi2ModelDescription = deserialize(&serialized_xml).unwrap();

    // Verify key fields are preserved
    assert_eq!(original_md.fmi_version, round_trip_md.fmi_version);
    assert_eq!(original_md.model_name, round_trip_md.model_name);
    assert_eq!(original_md.guid, round_trip_md.guid);
    assert_eq!(original_md.description, round_trip_md.description);
    assert_eq!(
        original_md.number_of_event_indicators,
        round_trip_md.number_of_event_indicators
    );
}

#[test]
#[cfg(feature = "fmi3")]
fn test_serialize_deserialize_fmi3_model_description() {
    // Read the FMI3 test file
    let test_file = std::env::current_dir()
        .map(|path| path.join("tests/FMI3.xml"))
        .unwrap();
    let xml_content = std::fs::read_to_string(test_file).unwrap();

    // Deserialize from XML
    let original_md: Fmi3ModelDescription = deserialize(&xml_content).unwrap();

    // Serialize back to XML
    let serialized_xml = serialize(&original_md, false).unwrap();

    // Deserialize again from the serialized XML
    let round_trip_md: Fmi3ModelDescription = deserialize(&serialized_xml).unwrap();

    // Verify key fields are preserved
    assert_eq!(original_md.fmi_version, round_trip_md.fmi_version);
    assert_eq!(original_md.model_name, round_trip_md.model_name);
    assert_eq!(
        original_md.instantiation_token,
        round_trip_md.instantiation_token
    );
    assert_eq!(original_md.description, round_trip_md.description);
}

#[test]
#[cfg(feature = "fmi2")]
fn test_serialize_fragment() {
    let unit = Fmi2Unit {
        name: "meter".to_string(),
        base_unit: Some(BaseUnit {
            m: Some(1),
            ..Default::default()
        }),
        display_unit: vec![],
    };

    // Test with XML declaration (fragment = false)
    let xml_with_declaration = serialize(&unit, false).unwrap();
    assert!(xml_with_declaration.starts_with("<?xml"));

    // Test without XML declaration (fragment = true)
    let xml_fragment = serialize(&unit, true).unwrap();
    assert!(!xml_fragment.starts_with("<?xml"));
    assert!(xml_fragment.contains("<Unit"));
}

#[test]
#[cfg(feature = "fmi2")]
fn test_serialize_deserialize_default_experiment() {
    let default_experiment = DefaultExperiment {
        start_time: 0.0,
        stop_time: 10.0,
        tolerance: 1e-7,
    };

    // Serialize to XML
    let xml = serialize(&default_experiment, true).unwrap();

    // Deserialize back
    let deserialized: DefaultExperiment = deserialize(&xml).unwrap();

    // Verify all fields are preserved
    assert_eq!(default_experiment.start_time, deserialized.start_time);
    assert_eq!(default_experiment.stop_time, deserialized.stop_time);
    assert_eq!(default_experiment.tolerance, deserialized.tolerance);
}

#[test]
fn test_deserialize_invalid_xml() {
    let invalid_xml = "<InvalidXml><UnclosedTag></InvalidXml>";

    #[cfg(feature = "fmi2")]
    {
        let result: Result<Fmi2ModelDescription, _> = deserialize(invalid_xml);
        assert!(result.is_err());

        if let Err(fmi_schema::Error::XmlParse(msg)) = result {
            assert!(!msg.is_empty());
        } else {
            panic!("Expected XmlParse error");
        }
    }

    #[cfg(feature = "fmi3")]
    {
        let result: Result<Fmi3ModelDescription, _> = deserialize(invalid_xml);
        assert!(result.is_err());

        if let Err(fmi_schema::Error::XmlParse(msg)) = result {
            assert!(!msg.is_empty());
        } else {
            panic!("Expected XmlParse error");
        }
    }
}

#[test]
#[cfg(feature = "fmi2")]
fn test_serialize_deserialize_unit() {
    let unit = Fmi2Unit {
        name: "kilogram".to_string(),
        base_unit: Some(BaseUnit {
            kg: Some(1),
            ..Default::default()
        }),
        display_unit: vec![],
    };

    // Serialize to XML fragment
    let xml = serialize(&unit, true).unwrap();
    assert!(xml.contains("kilogram"));
    assert!(xml.contains("kg=\"1\""));

    // Deserialize back
    let deserialized: Fmi2Unit = deserialize(&xml).unwrap();

    // Verify fields are preserved
    assert_eq!(unit.name, deserialized.name);
    assert_eq!(unit.base_unit, deserialized.base_unit);
    assert_eq!(unit.display_unit, deserialized.display_unit);
}

#[test]
#[cfg(feature = "fmi2")]
fn test_serialize_empty_structure() {
    // Test serialization of a structure with minimal data
    let base_unit = BaseUnit::default();

    // Serialize to XML
    let xml = serialize(&base_unit, true).unwrap();
    assert!(xml.contains("<BaseUnit"));

    // Deserialize back
    let deserialized: BaseUnit = deserialize(&xml).unwrap();
    assert_eq!(base_unit, deserialized);
}

#[test]
#[cfg(feature = "fmi2")]
fn test_fmi2_model_description_traits() {
    // Read the FMI2 test file
    let test_file = std::env::current_dir()
        .map(|path| path.join("tests/FMI2.xml"))
        .unwrap();
    let xml_content = std::fs::read_to_string(test_file).unwrap();
    let md: Fmi2ModelDescription = deserialize(&xml_content).unwrap();

    // Test FmiModelDescription trait
    assert_eq!(md.model_name(), "BouncingBall");
    assert_eq!(md.version_string(), "2.0");

    let version = md.version().unwrap();
    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 0);

    let major_version = md.major_version().unwrap();
    assert_eq!(major_version, fmi_schema::MajorVersion::FMI2);

    // Test DefaultExperiment trait
    assert!(md.start_time().is_some());
    assert!(md.stop_time().is_some());
    assert!(md.tolerance().is_some());
    assert_eq!(md.step_size(), None); // FMI2 doesn't have step_size in DefaultExperiment

    // Test round-trip serialization through trait
    let serialized = md.serialize().unwrap();
    let deserialized = Fmi2ModelDescription::deserialize(&serialized).unwrap();
    assert_eq!(md.model_name(), deserialized.model_name());
    assert_eq!(md.version_string(), deserialized.version_string());
}

#[test]
#[cfg(feature = "fmi2")]
fn test_fmi2_interface_types_traits() {
    // Read the FMI2 test file
    let test_file = std::env::current_dir()
        .map(|path| path.join("tests/FMI2.xml"))
        .unwrap();
    let xml_content = std::fs::read_to_string(test_file).unwrap();
    let md: Fmi2ModelDescription = deserialize(&xml_content).unwrap();

    // Test ModelExchange interface
    if let Some(me) = &md.model_exchange {
        assert_eq!(me.model_identifier(), "BouncingBall");
        assert_eq!(me.needs_execution_tool(), None);
        assert_eq!(me.can_be_instantiated_only_once_per_process(), None);
        assert_eq!(me.can_get_and_set_fmu_state(), Some(true));
        assert_eq!(me.can_serialize_fmu_state(), Some(true));
        assert_eq!(me.provides_directional_derivatives(), None);
        assert_eq!(me.provides_adjoint_derivatives(), None); // Not in FMI2
        assert_eq!(me.provides_per_element_dependencies(), None); // Not in FMI2
    }

    // Test CoSimulation interface
    if let Some(cs) = &md.co_simulation {
        assert_eq!(cs.model_identifier(), "BouncingBall");
        assert_eq!(cs.needs_execution_tool(), None);
        assert_eq!(cs.can_be_instantiated_only_once_per_process(), None);
        assert_eq!(cs.can_get_and_set_fmu_state(), Some(true));
        assert_eq!(cs.can_serialize_fmu_state(), Some(true));
        assert_eq!(cs.provides_directional_derivatives(), None);
        assert_eq!(cs.provides_adjoint_derivatives(), None); // Not in FMI2
        assert_eq!(cs.provides_per_element_dependencies(), None); // Not in FMI2
    }
}

#[test]
#[cfg(feature = "fmi3")]
fn test_fmi3_model_description_traits() {
    // Read the FMI3 test file
    let test_file = std::env::current_dir()
        .map(|path| path.join("tests/FMI3.xml"))
        .unwrap();
    let xml_content = std::fs::read_to_string(test_file).unwrap();
    let md: Fmi3ModelDescription = deserialize(&xml_content).unwrap();

    // Test FmiModelDescription trait
    assert_eq!(md.model_name(), "BouncingBall");
    assert_eq!(md.version_string(), "3.0-beta.2");

    let version = md.version().unwrap();
    assert_eq!(version.major, 3);
    assert_eq!(version.minor, 0);

    let major_version = md.major_version().unwrap();
    assert_eq!(major_version, fmi_schema::MajorVersion::FMI3);

    // Test round-trip serialization through trait
    let serialized = md.serialize().unwrap();
    let deserialized = Fmi3ModelDescription::deserialize(&serialized).unwrap();
    assert_eq!(md.model_name(), deserialized.model_name());
    assert_eq!(md.version_string(), deserialized.version_string());
}

#[test]
#[cfg(feature = "fmi3")]
fn test_fmi3_interface_types_traits() {
    // Read the FMI3 test file
    let test_file = std::env::current_dir()
        .map(|path| path.join("tests/FMI3.xml"))
        .unwrap();
    let xml_content = std::fs::read_to_string(test_file).unwrap();
    let md: Fmi3ModelDescription = deserialize(&xml_content).unwrap();

    // Test ModelExchange interface if present
    if let Some(me) = &md.model_exchange {
        assert_eq!(me.model_identifier(), "BouncingBall");
        // Test all FmiInterfaceType accessors
        let _ = me.needs_execution_tool();
        let _ = me.can_be_instantiated_only_once_per_process();
        let _ = me.can_get_and_set_fmu_state();
        let _ = me.can_serialize_fmu_state();
        let _ = me.provides_directional_derivatives();
        let _ = me.provides_adjoint_derivatives(); // Available in FMI3
        let _ = me.provides_per_element_dependencies(); // Available in FMI3
    }

    // Test CoSimulation interface if present
    if let Some(cs) = &md.co_simulation {
        assert_eq!(cs.model_identifier(), "BouncingBall");
        // Test all FmiInterfaceType accessors
        let _ = cs.needs_execution_tool();
        let _ = cs.can_be_instantiated_only_once_per_process();
        let _ = cs.can_get_and_set_fmu_state();
        let _ = cs.can_serialize_fmu_state();
        let _ = cs.provides_directional_derivatives();
        let _ = cs.provides_adjoint_derivatives(); // Available in FMI3
        let _ = cs.provides_per_element_dependencies(); // Available in FMI3
    }

    // Test ScheduledExecution interface if present (FMI3 only)
    if let Some(se) = &md.scheduled_execution {
        assert_eq!(se.model_identifier(), "BouncingBall");
        // Test all FmiInterfaceType accessors
        let _ = se.needs_execution_tool();
        let _ = se.can_be_instantiated_only_once_per_process();
        let _ = se.can_get_and_set_fmu_state();
        let _ = se.can_serialize_fmu_state();
        let _ = se.provides_directional_derivatives();
        let _ = se.provides_adjoint_derivatives(); // Available in FMI3
        let _ = se.provides_per_element_dependencies(); // Available in FMI3
    }
}

#[test]
#[cfg(feature = "fmi2")]
fn test_trait_error_handling() {
    // Create a model description with invalid version for testing error handling
    let mut md = Fmi2ModelDescription::default();
    md.model_name = "TestModel".to_string();
    md.fmi_version = "invalid.version".to_string();
    md.guid = "test-guid".to_string();

    // Test that version parsing fails gracefully
    assert!(md.version().is_err());
    assert!(md.major_version().is_err());

    // Test that other accessors still work
    assert_eq!(md.model_name(), "TestModel");
    assert_eq!(md.version_string(), "invalid.version");
}

#[test]
fn test_serialize_deserialize_round_trip_consistency() {
    // Test round-trip consistency for both FMI versions

    #[cfg(feature = "fmi2")]
    {
        let test_file = std::env::current_dir()
            .map(|path| path.join("tests/FMI2.xml"))
            .unwrap();
        let original_xml = std::fs::read_to_string(test_file).unwrap();
        let md: Fmi2ModelDescription = deserialize(&original_xml).unwrap();

        // Serialize using the trait method
        let trait_serialized = md.serialize().unwrap();
        let trait_deserialized = Fmi2ModelDescription::deserialize(&trait_serialized).unwrap();

        // Serialize using the standalone function
        let func_serialized = serialize(&md, false).unwrap();
        let func_deserialized: Fmi2ModelDescription = deserialize(&func_serialized).unwrap();

        // Both methods should produce equivalent results
        assert_eq!(
            trait_deserialized.model_name(),
            func_deserialized.model_name()
        );
        assert_eq!(
            trait_deserialized.version_string(),
            func_deserialized.version_string()
        );
        assert_eq!(trait_deserialized.guid, func_deserialized.guid);
    }

    #[cfg(feature = "fmi3")]
    {
        let test_file = std::env::current_dir()
            .map(|path| path.join("tests/FMI3.xml"))
            .unwrap();
        let original_xml = std::fs::read_to_string(test_file).unwrap();
        let md: Fmi3ModelDescription = deserialize(&original_xml).unwrap();

        // Serialize using the trait method
        let trait_serialized = md.serialize().unwrap();
        let trait_deserialized = Fmi3ModelDescription::deserialize(&trait_serialized).unwrap();

        // Serialize using the standalone function
        let func_serialized = serialize(&md, false).unwrap();
        let func_deserialized: Fmi3ModelDescription = deserialize(&func_serialized).unwrap();

        // Both methods should produce equivalent results
        assert_eq!(
            trait_deserialized.model_name(),
            func_deserialized.model_name()
        );
        assert_eq!(
            trait_deserialized.version_string(),
            func_deserialized.version_string()
        );
        assert_eq!(
            trait_deserialized.instantiation_token,
            func_deserialized.instantiation_token
        );
    }
}

#[test]
#[cfg(feature = "fmi2")]
fn test_fmi2_edge_cases_and_optional_fields() {
    // Create a minimal FMI2 model description to test edge cases
    let mut md = Fmi2ModelDescription::default();
    md.model_name = "MinimalModel".to_string();
    md.fmi_version = "2.0".to_string();
    md.guid = "minimal-guid".to_string();

    // Test with no default experiment (should return None values)
    assert_eq!(md.start_time(), None);
    assert_eq!(md.stop_time(), None);
    assert_eq!(md.tolerance(), None);
    assert_eq!(md.step_size(), None);

    // Test serialization/deserialization of minimal model
    let serialized = serialize(&md, false).unwrap();
    let deserialized: Fmi2ModelDescription = deserialize(&serialized).unwrap();

    assert_eq!(md.model_name(), deserialized.model_name());
    assert_eq!(md.version_string(), deserialized.version_string());
    assert_eq!(md.guid, deserialized.guid);

    // Verify trait behavior is consistent
    assert_eq!(md.major_version().unwrap(), fmi_schema::MajorVersion::FMI2);
}

#[test]
#[cfg(feature = "fmi3")]
fn test_fmi3_edge_cases_and_optional_fields() {
    // Create a minimal FMI3 model description to test edge cases
    let mut md = Fmi3ModelDescription::default();
    md.model_name = "MinimalModel".to_string();
    md.fmi_version = "3.0".to_string();
    md.instantiation_token = "minimal-token".to_string();

    // Test serialization/deserialization of minimal model
    let serialized = serialize(&md, false).unwrap();
    let deserialized: Fmi3ModelDescription = deserialize(&serialized).unwrap();

    assert_eq!(md.model_name(), deserialized.model_name());
    assert_eq!(md.version_string(), deserialized.version_string());
    assert_eq!(md.instantiation_token, deserialized.instantiation_token);

    // Verify trait behavior is consistent
    assert_eq!(md.major_version().unwrap(), fmi_schema::MajorVersion::FMI3);
}

#[test]
fn test_version_edge_cases() {
    #[cfg(feature = "fmi2")]
    {
        let mut md = Fmi2ModelDescription::default();
        md.model_name = "TestModel".to_string();
        md.guid = "test-guid".to_string();

        // Test various version strings
        md.fmi_version = "2.0.1".to_string();
        assert_eq!(md.version().unwrap().to_string(), "2.0.1");
        assert_eq!(md.major_version().unwrap(), fmi_schema::MajorVersion::FMI2);

        // Test with empty version (should fail)
        md.fmi_version = "".to_string();
        assert!(md.version().is_err());
        assert!(md.major_version().is_err());
    }
}

#[test]
#[cfg(all(feature = "fmi2", feature = "fmi3"))]
fn test_cross_version_compatibility() {
    // Test that FMI2 and FMI3 traits work consistently
    let fmi2_file = std::env::current_dir()
        .map(|path| path.join("tests/FMI2.xml"))
        .unwrap();
    let fmi2_xml = std::fs::read_to_string(fmi2_file).unwrap();
    let fmi2_md: Fmi2ModelDescription = deserialize(&fmi2_xml).unwrap();

    let fmi3_file = std::env::current_dir()
        .map(|path| path.join("tests/FMI3.xml"))
        .unwrap();
    let fmi3_xml = std::fs::read_to_string(fmi3_file).unwrap();
    let fmi3_md: Fmi3ModelDescription = deserialize(&fmi3_xml).unwrap();

    // Both should have the same model name (from test files)
    assert_eq!(fmi2_md.model_name(), fmi3_md.model_name());

    // Versions should be different
    assert_ne!(fmi2_md.version_string(), fmi3_md.version_string());
    assert_ne!(
        fmi2_md.major_version().unwrap(),
        fmi3_md.major_version().unwrap()
    );

    // Test that serialization works for both
    let fmi2_serialized = fmi2_md.serialize().unwrap();
    let fmi3_serialized = fmi3_md.serialize().unwrap();

    assert!(fmi2_serialized.contains("2.0"));
    assert!(fmi3_serialized.contains("3.0"));

    // Test round-trip consistency
    let fmi2_roundtrip = Fmi2ModelDescription::deserialize(&fmi2_serialized).unwrap();
    let fmi3_roundtrip = Fmi3ModelDescription::deserialize(&fmi3_serialized).unwrap();

    assert_eq!(fmi2_md.model_name(), fmi2_roundtrip.model_name());
    assert_eq!(fmi3_md.model_name(), fmi3_roundtrip.model_name());
}
