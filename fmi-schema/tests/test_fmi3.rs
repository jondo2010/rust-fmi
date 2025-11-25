//! Test FMI 3.0 schema by parsing the FMI3.xml file.

#[test]
#[cfg(feature = "fmi3")]
fn test_fmi3() {
    use fmi_schema::{
        fmi3::{
            BaseUnit, Causality, DependenciesKind, Float64Type, Fmi3ModelDescription, FmiFloat64,
            Initial, TypeDefinition, Variability, Variable,
        },
        traits::FmiInterfaceType,
        utils::AttrList,
    };

    let test_file = std::env::current_dir()
        .map(|path| path.join("tests/FMI3.xml"))
        .unwrap();
    let xml_content = std::fs::read_to_string(test_file).unwrap();
    let model: Fmi3ModelDescription = fmi_schema::deserialize(&xml_content).unwrap();

    let model_exchange = model.model_exchange.unwrap();
    assert_eq!(model_exchange.model_identifier(), "BouncingBall");
    assert_eq!(model_exchange.can_get_and_set_fmu_state(), Some(true));
    assert_eq!(model_exchange.can_serialize_fmu_state(), Some(true));

    let cosim = model.co_simulation.unwrap();
    assert_eq!(cosim.model_identifier(), "BouncingBall");
    assert_eq!(cosim.can_get_and_set_fmu_state(), Some(true));
    assert_eq!(cosim.can_serialize_fmu_state(), Some(true));
    assert_eq!(
        cosim.can_handle_variable_communication_step_size,
        Some(true)
    );
    assert_eq!(cosim.provides_intermediate_update, Some(true));
    assert_eq!(cosim.can_return_early_after_intermediate_update, Some(true));
    assert_eq!(cosim.fixed_internal_step_size, Some(1e-3));

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
    assert_eq!(
        type_defs.type_definitions,
        vec![
            TypeDefinition::Float64(Float64Type {
                name: "Position".to_string(),
                quantity: Some("Position".to_string()),
                unit: Some("m".to_string()),
                ..Default::default()
            }),
            TypeDefinition::Float64(Float64Type {
                name: "Velocity".to_string(),
                quantity: Some("Velocity".to_string()),
                unit: Some("m/s".to_string()),
                ..Default::default()
            }),
            TypeDefinition::Float64(Float64Type {
                name: "Acceleration".to_string(),
                quantity: Some("Acceleration".to_string()),
                unit: Some("m/s2".to_string()),
                ..Default::default()
            }),
        ]
    );

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
    assert_eq!(model_vars.variables.len(), 7);
    assert_eq!(
        model_vars.variables[4],
        Variable::Float64(FmiFloat64 {
            name: "der(v)".to_owned(),
            description: Some("Derivative of v".to_owned()),
            causality: Causality::Local,
            value_reference: 4,
            variability: Some(Variability::Continuous),
            declared_type: Some("Acceleration".to_owned()),
            initial: Some(Initial::Calculated),
            derivative: Some(3),
            ..Default::default()
        })
    );
    assert_eq!(
        model_vars.variables[6],
        Variable::Float64(FmiFloat64 {
            name: "e".to_owned(),
            value_reference: 6,
            causality: Causality::Parameter,
            variability: Some(Variability::Tunable),
            initial: Some(Initial::Exact),
            description: Some("Coefficient of restitution".to_owned()),
            start: Some(AttrList(vec![0.7])),
            min: Some(0.5),
            max: Some(1.0),
            ..Default::default()
        })
    );

    let model_structure = &model.model_structure;
    let outputs = model_structure.outputs().collect::<Vec<_>>();
    assert_eq!(outputs.len(), 2);
    assert_eq!(outputs[0].value_reference, 1);
    assert_eq!(outputs[1].value_reference, 3);

    let ders = model_structure
        .continuous_state_derivatives()
        .collect::<Vec<_>>();
    assert_eq!(ders.len(), 2);
    let initials = model_structure.initial_unknowns().collect::<Vec<_>>();
    assert_eq!(initials.len(), 2);
    assert_eq!(initials[0].value_reference, 2);
    assert_eq!(initials[0].dependencies, Some(AttrList(vec![3])));
    assert_eq!(
        initials[0].dependencies_kind,
        Some(AttrList(vec![DependenciesKind::Constant]))
    );
    let event_indicators = model_structure.event_indicators().collect::<Vec<_>>();
    assert_eq!(event_indicators.len(), 1);
}
