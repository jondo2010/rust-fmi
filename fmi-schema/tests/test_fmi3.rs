//! Test FMI 3.0 schema by parsing the FMI3.xml file.

#[test]
#[cfg(feature = "fmi3")]
fn test_fmi3() {
    use fmi_schema::fmi3::{
        AbstractVariableTrait, BaseTypeTrait, BaseUnit, DependenciesKind, Fmi3ModelExchange,
        FmiModelDescription, Variability,
    };

    let test_file = std::env::current_dir()
        .map(|path| path.join("tests/FMI3.xml"))
        .unwrap();
    let file = std::fs::File::open(test_file).unwrap();
    let buf_reader = std::io::BufReader::new(file);
    let model: FmiModelDescription = yaserde::de::from_reader(buf_reader).unwrap();

    let model_exchange = model.model_exchange.unwrap();
    assert_eq!(
        model_exchange,
        Fmi3ModelExchange {
            model_identifier: "BouncingBall".to_owned(),
            can_get_and_set_fmu_state: Some(true),
            can_serialize_fmu_state: Some(true),
            ..Default::default()
        }
    );

    let unit_defs = model.unit_definitions.unwrap();
    assert_eq!(unit_defs.units.len(), 3);
    assert_eq!(unit_defs.units[0].name, "m");
    assert_eq!(
        unit_defs.units[1].base_unit,
        Some(BaseUnit {
            m: Some(1),
            s: Some(-1),
            ..Default::default()
        })
    );

    let type_defs = model.type_definitions.unwrap();
    assert_eq!(type_defs.float64types.len(), 3);
    assert_eq!(type_defs.float64types[0].name(), "Position");
    //Float64Type { name: "Position", quantity: "Position", unit: "m" },

    let log_cats = model.log_categories.unwrap();
    assert_eq!(log_cats.categories.len(), 2);
    assert_eq!(&log_cats.categories[0].name, "logEvents");
    assert_eq!(
        log_cats.categories[0].description.as_deref(),
        Some("Log events")
    );

    let default_exp = model.default_experiment.unwrap();
    assert_eq!(default_exp.start_time, Some(0.0));
    assert_eq!(default_exp.stop_time, Some(3.0));
    assert_eq!(default_exp.step_size, Some(1e-3));

    let model_vars = &model.model_variables;
    assert_eq!(model_vars.float64.len(), 7);
    assert_eq!(model_vars.float64[4].name(), "der(v)");
    assert_eq!(model_vars.float64[4].value_reference(), 4);
    assert_eq!(model_vars.float64[4].variability(), Variability::Continuous);

    let model_structure = &model.model_structure;
    assert_eq!(model_structure.outputs.len(), 2);
    assert_eq!(model_structure.outputs[0].value_reference, 1);
    assert_eq!(model_structure.outputs[1].value_reference, 3);
    assert_eq!(model_structure.continuous_state_derivative.len(), 2);
    assert_eq!(model_structure.initial_unknown.len(), 2);
    assert_eq!(model_structure.initial_unknown[0].value_reference, 2);
    assert_eq!(model_structure.initial_unknown[0].dependencies, vec![3]);
    assert_eq!(
        model_structure.initial_unknown[0].dependencies_kind,
        vec![DependenciesKind::Constant]
    );
    assert_eq!(model_structure.event_indicator.len(), 1);
}
