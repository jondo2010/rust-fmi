//! Focused public-API tests for schema utility and variable-count behavior.

use fmi_schema::variable_counts::VariableCounts;

#[test]
#[cfg(feature = "fmi2")]
fn fmi2_model_counts_aggregate_type_variability_and_causality() {
    let xml = r#"<ModelVariables>
        <ScalarVariable name="real-parameter" valueReference="1" causality="parameter" variability="constant"><Real/></ScalarVariable>
        <ScalarVariable name="real-calculated" valueReference="2" causality="calculatedParameter" variability="continuous"><Real/></ScalarVariable>
        <ScalarVariable name="integer-input" valueReference="3" causality="input" variability="discrete"><Integer/></ScalarVariable>
        <ScalarVariable name="boolean-output" valueReference="4" causality="output" variability="fixed"><Boolean/></ScalarVariable>
        <ScalarVariable name="string-local" valueReference="5" causality="local" variability="tunable"><String/></ScalarVariable>
        <ScalarVariable name="enum-independent" valueReference="6" causality="independent" variability="continuous"><Enumeration/></ScalarVariable>
    </ModelVariables>"#;
    let variables: fmi_schema::fmi2::ModelVariables = fmi_schema::deserialize(xml).unwrap();

    let counts = variables.model_counts();

    assert_eq!(counts.num_constants, 1);
    assert_eq!(counts.num_parameters, 1);
    assert_eq!(counts.num_discrete, 1);
    assert_eq!(counts.num_continuous, 2);
    assert_eq!(counts.num_inputs, 1);
    assert_eq!(counts.num_outputs, 1);
    assert_eq!(counts.num_local, 1);
    assert_eq!(counts.num_independent, 1);
    assert_eq!(counts.num_calculated_parameters, 1);
    assert_eq!(counts.num_real_vars, 2);
    assert_eq!(counts.num_integer_vars, 1);
    assert_eq!(counts.num_enum_vars, 1);
    assert_eq!(counts.num_bool_vars, 1);
    assert_eq!(counts.num_string_vars, 1);
}

#[test]
#[cfg(feature = "fmi3")]
fn fmi3_model_counts_aggregate_type_variability_and_causality() {
    let xml = r#"<ModelVariables>
        <Float32 name="float32-parameter" valueReference="1" causality="parameter" variability="constant"/>
        <Float64 name="float64-calculated" valueReference="2" causality="calculatedParameter" variability="continuous"/>
        <Int8 name="int8-input" valueReference="3" causality="input" variability="discrete"/>
        <UInt8 name="uint8-output" valueReference="4" causality="output" variability="fixed"/>
        <Int16 name="int16-local" valueReference="5" causality="local" variability="tunable"/>
        <UInt16 name="uint16-independent" valueReference="6" causality="independent" variability="discrete"/>
        <Int32 name="int32-dependent" valueReference="7" causality="dependent" variability="discrete"/>
        <UInt32 name="uint32-structural" valueReference="8" causality="structuralParameter" variability="fixed"/>
        <Int64 name="int64" valueReference="9"/>
        <UInt64 name="uint64" valueReference="10"/>
        <Boolean name="boolean" valueReference="11"/>
        <String name="string" valueReference="12"/>
        <Binary name="binary" valueReference="13"/>
        <Clock name="clock" valueReference="14"/>
    </ModelVariables>"#;
    let variables: fmi_schema::fmi3::ModelVariables = fmi_schema::deserialize(xml).unwrap();

    let counts = variables.model_counts();

    assert_eq!(counts.num_constants, 1);
    assert_eq!(counts.num_parameters, 1);
    assert_eq!(counts.num_discrete, 9);
    assert_eq!(counts.num_continuous, 1);
    assert_eq!(counts.num_inputs, 1);
    assert_eq!(counts.num_outputs, 1);
    assert_eq!(counts.num_local, 7);
    assert_eq!(counts.num_independent, 1);
    assert_eq!(counts.num_calculated_parameters, 1);
    assert_eq!(counts.num_real_vars, 2);
    assert_eq!(counts.num_integer_vars, 8);
    assert_eq!(counts.num_enum_vars, 0);
    assert_eq!(counts.num_bool_vars, 1);
    assert_eq!(counts.num_string_vars, 1);
}
