//! Test `fmi-sim` against the reference FMUs.

#![allow(unused_imports)]
#![allow(dead_code)]

use std::{path::PathBuf, str::FromStr};

use arrow::{
    array::{AsArray, Float64Array},
    datatypes::{
        ArrowPrimitiveType, Float32Type, Float64Type, Int16Type, Int32Type, Int64Type, Int8Type,
        UInt16Type, UInt32Type, UInt64Type, UInt8Type,
    },
};
use fmi_sim::options::{
    CoSimulationOptions, CommonOptions, FmiSimOptions, Interface, ModelExchangeOptions,
};

#[test]
fn test_start_time() {
    let mut ref_fmus = fmi_test_data::ReferenceFmus::new().unwrap();
    let import = ref_fmus.get_reference_fmu_fmi3("BouncingBall").unwrap();

    let options = CoSimulationOptions {
        common: CommonOptions {
            start_time: Some(0.5),
            ..Default::default()
        },
        ..Default::default()
    };
    let output = fmi_sim::sim::fmi3::co_simulation(&import, options, None).unwrap();

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
    let import = ref_fmus.get_reference_fmu_fmi3("BouncingBall").unwrap();

    let options = CoSimulationOptions {
        common: CommonOptions {
            stop_time: Some(0.5),
            ..Default::default()
        },
        ..Default::default()
    };
    let output = fmi_sim::sim::fmi3::co_simulation(&import, options, None).unwrap();

    let time = output
        .column_by_name("time")
        .unwrap()
        .as_primitive::<Float64Type>();
    assert_eq!(time.value(time.len() - 1), 0.5);
}

#[test_log::test]
fn test_start_value_types() {
    let mut ref_fmus = fmi_test_data::ReferenceFmus::new().unwrap();
    let import = ref_fmus.get_reference_fmu_fmi3("Feedthrough").unwrap();

    let common = CommonOptions {
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
    let options = CoSimulationOptions {
        common,
        event_mode_used: false,
        early_return_allowed: false,
    };
    let output = fmi_sim::sim::fmi3::co_simulation(&import, options, None).unwrap();

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

fn compare_record_batches(
    expected: &arrow::record_batch::RecordBatch,
    actual: &arrow::record_batch::RecordBatch,
) {
    // Compare the schema
    assert_eq!(expected.schema().fields(), actual.schema().fields());

    // Compare the columns
    for (i, (expected_column, actual_column)) in
        expected.columns().iter().zip(actual.columns()).enumerate()
    {
        assert_eq!(
            expected_column,
            actual_column,
            "Column '{}' does not match",
            expected.schema().field(i).name()
        );
    }
}

fn compare_f64_column_by_name(
    expected: &arrow::record_batch::RecordBatch,
    actual: &arrow::record_batch::RecordBatch,
    column_name: &str,
) {
    let expected = expected
        .column_by_name(column_name)
        .unwrap()
        .as_primitive::<Float64Type>();
    let actual = actual
        .column_by_name(column_name)
        .unwrap()
        .as_primitive::<Float64Type>();

    for (e, a) in expected.iter().zip(actual.iter()) {
        float_cmp::assert_approx_eq!(f64, e.unwrap(), a.unwrap(), epsilon = 1e-9, ulps = 1);
    }
}

#[test_log::test]
#[cfg(feature = "disable")]
fn test_bouncing_ball() {
    let mut ref_fmus = fmi_test_data::ReferenceFmus::new().unwrap();
    let model = ref_fmus.extract_reference_fmu("BouncingBall", 3).unwrap();

    for (iface, options, expected) in [
        (
            "CS",
            FmiSimOptions {
                interface: Interface::CoSimulation(CoSimulationOptions::default()),
                model: model.clone(),
            },
            fmi_sim::sim::util::read_csv("tests/data/bouncing_ball_cs_expected.csv")
                .expect("Error reading expected output"),
        ),
        (
            "ME",
            FmiSimOptions {
                interface: Interface::ModelExchange(ModelExchangeOptions::default()),
                model: model.clone(),
            },
            fmi_sim::sim::util::read_csv("tests/data/bouncing_ball_me_expected.csv")
                .expect("Error reading expected output"),
        ),
    ] {
        let output = fmi_sim::simulate(options).expect("Error simulating FMU");

        // Compare the schema
        assert_eq!(output.schema().fields(), expected.schema().fields());

        log::info!("{iface}: Comparing 'time' column");
        compare_f64_column_by_name(&expected, &output, "time");
        log::info!("{iface}: Comparing 'h' column");
        compare_f64_column_by_name(&expected, &output, "h");
        log::info!("{iface}: Comparing 'v' column");
        compare_f64_column_by_name(&expected, &output, "v");
    }
}

#[cfg(feature = "broken")]
#[test_log::test]
fn test_input_file() {
    let mut ref_fmus = fmi_test_data::ReferenceFmus::new().unwrap();
    let import = ref_fmus.get_reference_fmu_fmi3("Feedthrough").unwrap();

    let options = CoSimulationOptions {
        common: CommonOptions {
            stop_time: Some(5.0),
            ..Default::default()
        },
        event_mode_used: false,
        early_return_allowed: false,
    };

    let input_data = fmi_sim::sim::util::read_csv("tests/data/feedthrough_in.csv")
        .expect("Error reading input data");

    let output = fmi_sim::sim::fmi3::co_simulation(&import, options, Some(input_data)).unwrap();

    let f64_cts_out = output
        .column_by_name("Float64_continuous_output")
        .unwrap()
        .as_primitive::<Float64Type>();

    // start value
    assert_eq!(f64_cts_out.value(0), 3.0);

    // extrapolation
    assert_eq!(f64_cts_out.value(f64_cts_out.len() - 1), 3.0);
}
