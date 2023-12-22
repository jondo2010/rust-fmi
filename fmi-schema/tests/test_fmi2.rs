//! Test FMI 2.0 schema by parsing the FMI2.xml file.

use fmi_schema::fmi2::{BaseUnit, FmiModelDescription, SimpleTypeElement};

#[test]
#[cfg(feature = "fmi2")]
fn test_fmi2() {
    let test_file = std::env::current_dir()
        .map(|path| path.join("tests/FMI2.xml"))
        .unwrap();
    let file = std::fs::File::open(test_file).unwrap();
    let buf_reader = std::io::BufReader::new(file);
    let md: FmiModelDescription = yaserde::de::from_reader(buf_reader).unwrap();

    assert_eq!(md.fmi_version, "2.0");
    assert_eq!(md.model_name, "BouncingBall");
    assert_eq!(
        md.description.as_deref(),
        Some("This model calculates the trajectory, over time, of a ball dropped from a height of 1 m.")
    );
    assert_eq!(md.guid, "{8c4e810f-3df3-4a00-8276-176fa3c9f003}");
    assert_eq!(md.number_of_event_indicators, 1);

    let me = md.model_exchange.unwrap();
    assert_eq!(me.model_identifier, "BouncingBall");
    assert_eq!(me.can_not_use_memory_management_functions, true);
    assert_eq!(me.can_get_and_set_fmu_state, true);
    assert_eq!(me.can_serialize_fmu_state, true);
    assert_eq!(me.source_files.files.len(), 1);
    assert_eq!(me.source_files.files[0].name, "all.c");

    let cs = md.co_simulation.unwrap();
    assert_eq!(cs.model_identifier, "BouncingBall");
    assert_eq!(cs.can_handle_variable_communication_step_size, true);
    assert_eq!(cs.can_not_use_memory_management_functions, true);
    assert_eq!(cs.can_get_and_set_fmu_state, true);
    assert_eq!(cs.can_serialize_fmu_state, true);
    assert_eq!(cs.source_files.files.len(), 1);
    assert_eq!(cs.source_files.files[0].name, "all.c");

    let units = md.unit_definitions.unwrap();
    assert_eq!(units.units.len(), 3);
    assert_eq!(units.units[0].name, "m");
    assert_eq!(
        units.units[0].base_unit,
        Some(BaseUnit {
            m: Some(1),
            ..Default::default()
        })
    );

    let typedefs = md.type_definitions.unwrap();
    assert_eq!(typedefs.types.len(), 3);
    assert_eq!(typedefs.types[0].name, "Position");
    assert!(matches!(typedefs.types[0].elem, SimpleTypeElement::Real(_)));
}
