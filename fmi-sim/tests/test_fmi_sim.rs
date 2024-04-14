//! Test `fmi-sim` against the reference FMUs.

#![allow(unused_imports)]
#![allow(dead_code)]

use std::{io::Cursor, path::PathBuf, str::FromStr};

use arrow::{
    array::{AsArray, Float64Array},
    datatypes::{
        ArrowPrimitiveType, Float32Type, Float64Type, Int16Type, Int32Type, Int64Type, Int8Type,
        UInt16Type, UInt32Type, UInt64Type, UInt8Type,
    },
};
use fmi::{fmi2::import::Fmi2Import, fmi3::import::Fmi3Import, schema::MajorVersion};
use fmi_sim::{
    options::{CoSimulationOptions, CommonOptions, FmiSimOptions, Interface, ModelExchangeOptions},
    sim::traits::FmiSim,
};

#[rstest::fixture]
fn ref_fmus() -> fmi_test_data::ReferenceFmus {
    fmi_test_data::ReferenceFmus::new().unwrap()
}

#[rstest::rstest]
#[case::cs(Interface::CoSimulation(CoSimulationOptions {common: CommonOptions { start_time: Some(0.5), output_interval: Some(0.1), ..Default::default() }, ..Default::default()}))]
#[case::me(Interface::ModelExchange(ModelExchangeOptions {common: CommonOptions { start_time: Some(0.5), output_interval: Some(0.1), ..Default::default() }, ..Default::default()}))]
#[trace]
#[test_log::test]
fn test_start_time(
    mut ref_fmus: fmi_test_data::ReferenceFmus,
    #[values(MajorVersion::FMI2, MajorVersion::FMI3)] fmi_version: MajorVersion,
    #[case] interface: Interface,
) {
    let fmu_file = ref_fmus
        .extract_reference_fmu("BouncingBall", fmi_version)
        .unwrap();

    let options = FmiSimOptions {
        interface,
        model: fmu_file.path().to_path_buf(),
        ..Default::default()
    };

    let (output, _) = fmi_sim::simulate(&options).unwrap();
    assert_eq!(
        output
            .column_by_name("time")
            .unwrap()
            .as_primitive::<Float64Type>()
            .value(0),
        0.5,
    );
}

#[rstest::rstest]
#[case::cs(Interface::CoSimulation(CoSimulationOptions {common: CommonOptions { stop_time: Some(0.5), output_interval: Some(0.1), ..Default::default() }, ..Default::default()}))]
#[case::me(Interface::ModelExchange(ModelExchangeOptions {common: CommonOptions { stop_time: Some(0.5), output_interval: Some(0.1), ..Default::default() }, ..Default::default()}))]
#[trace]
#[test_log::test]
fn test_stop_time(
    mut ref_fmus: fmi_test_data::ReferenceFmus,
    #[values(MajorVersion::FMI2, MajorVersion::FMI3)] fmi_version: MajorVersion,
    #[case] interface: Interface,
) {
    let fmu_file = ref_fmus
        .extract_reference_fmu("BouncingBall", fmi_version)
        .unwrap();

    let options = FmiSimOptions {
        interface,
        model: fmu_file.path().to_path_buf(),
        ..Default::default()
    };

    let (output, _) = fmi_sim::simulate(&options).unwrap();

    let time = output
        .column_by_name("time")
        .unwrap()
        .as_primitive::<Float64Type>();
    assert_eq!(time.value(time.len() - 1), 0.5,);
}

#[test_log::test]
fn test_start_value_types() {
    let mut ref_fmus = fmi_test_data::ReferenceFmus::new().unwrap();
    let import: Fmi3Import = ref_fmus.get_reference_fmu("Feedthrough").unwrap();

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
    let (output, _) = import.simulate_cs(&options, None).unwrap();

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

#[cfg(feature = "fails")]
#[test_log::test]
fn test_bouncing_ball() {
    let mut ref_fmus = fmi_test_data::ReferenceFmus::new().unwrap();
    let model = ref_fmus.extract_reference_fmu("BouncingBall", 3).unwrap();

    for (iface, options, expected) in [
        (
            "CS",
            FmiSimOptions {
                interface: Interface::CoSimulation(CoSimulationOptions::default()),
                model: model.clone(),
                ..Default::default()
            },
            fmi_sim::sim::util::read_csv("tests/data/bouncing_ball_cs_expected.csv")
                .expect("Error reading expected output"),
        ),
        (
            "ME",
            FmiSimOptions {
                interface: Interface::ModelExchange(ModelExchangeOptions::default()),
                model: model.clone(),
                ..Default::default()
            },
            fmi_sim::sim::util::read_csv("tests/data/bouncing_ball_me_expected.csv")
                .expect("Error reading expected output"),
        ),
    ] {
        let output = fmi_sim::simulate(&options).expect("Error simulating FMU");

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

#[rstest::fixture]
fn input_data() -> arrow::record_batch::RecordBatch {
    fmi_sim::sim::util::read_csv_file("tests/data/feedthrough_in.csv")
        .expect("Error reading input data")
}

#[rstest::fixture]
fn feedthrough_output_data(
    #[default(Interface::CoSimulation(Default::default()))] iface: Interface,
) -> arrow::record_batch::RecordBatch {
    let expected = match iface {
        Interface::ModelExchange(..) => {
            r#""time","Float32_continuous_output","Float32_discrete_output","Float64_continuous_output","Float64_discrete_output","Int8_output","UInt8_output","Int16_output","UInt16_output","Int32_output","UInt32_output","Int64_output","UInt64_output","Boolean_output","Binary_output","Enumeration_output"
0,0,0,3,3,0,0,0,0,1,0,0,0,0,666f6f,1
1,0,0,3,3,0,0,0,0,1,0,0,0,0,666f6f,1
1,0,0,2,2,0,0,0,0,1,0,0,0,0,666f6f,1
2,0,0,3,2,0,0,0,0,1,0,0,0,0,666f6f,1
2,0,0,3,3,0,0,0,0,1,0,0,0,0,666f6f,1
3,0,0,3,3,0,0,0,0,1,0,0,0,0,666f6f,1
3,0,0,3,3,0,0,0,0,2,0,0,0,0,666f6f,1
4,0,0,3,3,0,0,0,0,2,0,0,0,0,666f6f,1
5,0,0,3,3,0,0,0,0,2,0,0,0,0,666f6f,1"#
        }
        Interface::CoSimulation(..) => {
            r#""time","Float32_continuous_output","Float32_discrete_output","Float64_continuous_output","Float64_discrete_output","Int8_output","UInt8_output","Int16_output","UInt16_output","Int32_output","UInt32_output","Int64_output","UInt64_output","Boolean_output","Binary_output","Enumeration_output"
0,0,0,3,3,0,0,0,0,1,0,0,0,0,666f6f,1
1,0,0,3,3,0,0,0,0,1,0,0,0,0,666f6f,1
2,0,0,2,2,0,0,0,0,1,0,0,0,0,666f6f,1
3,0,0,3,3,0,0,0,0,1,0,0,0,0,666f6f,1
4,0,0,3,3,0,0,0,0,2,0,0,0,0,666f6f,1
5,0,0,3,3,0,0,0,0,2,0,0,0,0,666f6f,1"#
        }
    };
    let mut cur = Cursor::new(expected);
    fmi_sim::sim::util::read_csv(&mut cur).expect("Error reading output data")
}

#[rstest::rstest]
#[case::cs(Interface::CoSimulation(CoSimulationOptions {common: CommonOptions { stop_time: Some(5.0), output_interval: Some(1.0), ..Default::default() }, ..Default::default()}))]
#[case::me(Interface::ModelExchange(ModelExchangeOptions {common: CommonOptions { stop_time: Some(5.0), output_interval: Some(1.0), ..Default::default() }, ..Default::default()}))]
//#[trace]
#[test_log::test]
fn test_input_data(
    mut ref_fmus: fmi_test_data::ReferenceFmus,
    #[values(MajorVersion::FMI2, MajorVersion::FMI3)] fmi_version: MajorVersion,
    #[case] interface: Interface,
    input_data: arrow::record_batch::RecordBatch,
    //#[with(interface)] feedthrough_output_data: arrow::record_batch::RecordBatch,
) {
    let (output, _) = match fmi_version {
        MajorVersion::FMI1 => unimplemented!(),
        MajorVersion::FMI2 => {
            let import: Fmi2Import = ref_fmus.get_reference_fmu("Feedthrough").unwrap();
            fmi_sim::sim::simulate_with(Some(input_data), &interface, import).unwrap()
        }
        MajorVersion::FMI3 => {
            let import: Fmi3Import = ref_fmus.get_reference_fmu("Feedthrough").unwrap();
            fmi_sim::sim::simulate_with(Some(input_data), &interface, import).unwrap()
        }
    };

    // Pretty-print the output
    println!(
        "Outputs:\n{}",
        arrow::util::pretty::pretty_format_batches(&[output.clone()]).unwrap()
    );

    let f64_cts_out = output
        .column_by_name("Float64_continuous_output")
        .unwrap()
        .as_primitive::<Float64Type>();

    // start value
    assert_eq!(f64_cts_out.value(0), 3.0);

    // extrapolation
    assert_eq!(f64_cts_out.value(f64_cts_out.len() - 1), 3.0);
}
