//! Test `fmi-sim` against the reference FMUs.

use std::{path::PathBuf, str::FromStr};

use arrow::{
    array::AsArray,
    datatypes::{
        Float32Type, Float64Type, Int16Type, Int32Type, Int64Type, Int8Type, UInt16Type,
        UInt32Type, UInt64Type, UInt8Type,
    },
};
use fmi_sim::{options::FmiCheckOptions, sim::options::SimOptions};

#[test]
fn test_start_time() {
    let mut ref_fmus = fmi_test_data::ReferenceFmus::new().unwrap();
    let model = ref_fmus.extract_reference_fmu("BouncingBall", 3).unwrap();

    let opts = SimOptions {
        start_time: Some(0.5),
        ..Default::default()
    };
    let options = FmiCheckOptions {
        model,
        action: fmi_sim::options::Action::CS(opts),
    };
    let output = fmi_sim::simulate(options).expect("Error simulating FMU");

    assert_eq!(
        output
            .column_by_name("time")
            .unwrap()
            .as_primitive::<Float64Type>()
            .value(0),
        0.5
    );
}

#[test_log::test]
fn test_stop_time() {
    let mut ref_fmus = fmi_test_data::ReferenceFmus::new().unwrap();
    let model = ref_fmus.extract_reference_fmu("BouncingBall", 3).unwrap();
    let opts = SimOptions {
        stop_time: Some(0.5),
        ..Default::default()
    };
    let options = FmiCheckOptions {
        model,
        action: fmi_sim::options::Action::CS(opts),
    };
    let output = fmi_sim::simulate(options).expect("Error simulating FMU");

    let time = output
        .column_by_name("time")
        .unwrap()
        .as_primitive::<Float64Type>();
    assert_eq!(time.value(time.len() - 1), 0.5);
}

#[test_log::test]
fn test_start_value_types() {
    let mut ref_fmus = fmi_test_data::ReferenceFmus::new().unwrap();
    let model = ref_fmus.extract_reference_fmu("Feedthrough", 3).unwrap();
    let simulate = SimOptions {
        initial_values: [
            "Float64_continuous_input=-5e-1",
            "Int32_input=2147483647",
            "Boolean_input=1",
            "String_parameter='FMI is awesome!'",
            //"Enumeration_input=2",
            "Float32_continuous_input=0.2",
            "Int8_input=127",
            "UInt8_input=255",
            "Int16_input=32767",
            "UInt16_input=65535",
            "UInt32_input=4294967295",
            "Int64_input=9223372036854775807",
            "UInt64_input=18446744073709551615",
            "Binary_input=42696E617279",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect(),
        step_size: Some(1.0),
        ..Default::default()
    };
    let options = FmiCheckOptions {
        model,
        action: fmi_sim::options::Action::CS(simulate),
    };

    let output = fmi_sim::simulate(options).expect("Error simulating FMU");

    assert_eq!(
        output
            .column_by_name("Float64_continuous_output")
            .unwrap()
            .as_primitive::<Float64Type>()
            .value(0),
        -0.5
    );
    assert_eq!(
        output
            .column_by_name("Int32_output")
            .unwrap()
            .as_primitive::<Int32Type>()
            .value(0),
        2147483647
    );
    assert_eq!(
        output
            .column_by_name("Boolean_output")
            .unwrap()
            .as_boolean()
            .value(0),
        true
    );
    assert_eq!(
        output
            .column_by_name("Float32_continuous_output")
            .unwrap()
            .as_primitive::<Float32Type>()
            .value(0),
        0.2
    );
    assert_eq!(
        output
            .column_by_name("Int8_output")
            .unwrap()
            .as_primitive::<Int8Type>()
            .value(0),
        127
    );
    assert_eq!(
        output
            .column_by_name("UInt8_output")
            .unwrap()
            .as_primitive::<UInt8Type>()
            .value(0),
        255
    );
    assert_eq!(
        output
            .column_by_name("Int16_output")
            .unwrap()
            .as_primitive::<Int16Type>()
            .value(0),
        32767
    );
    assert_eq!(
        output
            .column_by_name("UInt16_output")
            .unwrap()
            .as_primitive::<UInt16Type>()
            .value(0),
        65535
    );
    assert_eq!(
        output
            .column_by_name("UInt32_output")
            .unwrap()
            .as_primitive::<UInt32Type>()
            .value(0),
        4294967295
    );
    assert_eq!(
        output
            .column_by_name("Int64_output")
            .unwrap()
            .as_primitive::<Int64Type>()
            .value(0),
        9223372036854775807
    );
    assert_eq!(
        output
            .column_by_name("UInt64_output")
            .unwrap()
            .as_primitive::<UInt64Type>()
            .value(0),
        18446744073709551615
    );
    assert_eq!(
        output
            .column_by_name("Binary_output")
            .unwrap()
            .as_binary::<i32>()
            .value(0),
        b"42696E617279"
    );
}

#[test_log::test]
fn test_input_file() {
    let mut ref_fmus = fmi_test_data::ReferenceFmus::new().unwrap();
    let model = ref_fmus.extract_reference_fmu("Feedthrough", 3).unwrap();
    let simulate = SimOptions {
        input_file: Some(PathBuf::from_str("tests/data/feedthrough_in.csv").unwrap()),
        stop_time: Some(5.0),
        ..Default::default()
    };
    let options = FmiCheckOptions {
        model: model,
        action: fmi_sim::options::Action::CS(simulate),
    };
    let output = fmi_sim::simulate(options).expect("Error simulating FMU");

    let f64_cts_out = output
        .column_by_name("Float64_continuous_output")
        .unwrap()
        .as_primitive::<Float64Type>();

    // start value
    assert_eq!(f64_cts_out.value(0), 3.0);

    // extrapolation
    assert_eq!(f64_cts_out.value(f64_cts_out.len() - 1), 3.0);
}
