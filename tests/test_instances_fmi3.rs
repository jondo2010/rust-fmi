//! Test the FMI3.0 instance API.

use fmi::{
    fmi3::{Common, Fmi3Model, GetSet, ModelExchange, import::Fmi3Import},
    schema::fmi3::{AbstractVariableTrait, InitializableVariableTrait},
    traits::FmiImport as _,
};
use fmi_test_data::ReferenceFmus;

extern crate fmi;
extern crate fmi_test_data;

#[test]
fn test_instance_dahlquist() {
    let mut ref_fmus = ReferenceFmus::new().unwrap();
    let import: Fmi3Import = ref_fmus.get_reference_fmu("Dahlquist").unwrap();
    let mut inst1 = import.instantiate_me("inst1", true, true).unwrap();
    assert_eq!(inst1.get_version(), "3.0");
    let log_cats: Vec<_> = import
        .model_description()
        .log_categories
        .as_ref()
        .unwrap()
        .categories
        .iter()
        .map(|x| x.name.as_str())
        .collect();
    inst1.set_debug_logging(true, &log_cats).unwrap();

    inst1.enter_configuration_mode().ok().unwrap();
    inst1.exit_configuration_mode().ok().unwrap();

    inst1
        .enter_initialization_mode(None, 0.0, None)
        .ok()
        .unwrap();
    inst1.exit_initialization_mode().ok().unwrap();
    inst1.set_time(1234.0).ok().unwrap();

    inst1.enter_continuous_time_mode().ok().unwrap();

    let states = (0..import
        .model_description()
        .model_structure
        .continuous_state_derivative
        .len())
        .map(|x| x as f64)
        .collect::<Vec<_>>();

    inst1.set_continuous_states(&states).ok().unwrap();
    let mut enter_event_mode = false;
    let mut terminate_simulation = false;
    inst1
        .completed_integrator_step(false, &mut enter_event_mode, &mut terminate_simulation)
        .ok()
        .unwrap();
    assert!(!enter_event_mode);
    assert!(!terminate_simulation);

    let mut ders = vec![0.0; states.len()];
    inst1
        .get_continuous_state_derivatives(ders.as_mut_slice())
        .ok()
        .unwrap();
    assert_eq!(ders, vec![-0.0]);
}

/// Test the get/set interface on strings variables with the `Feedthrough` FMU
#[test]
fn test_instance_feedthrough_string() {
    let mut ref_fmus = ReferenceFmus::new().unwrap();
    let import: Fmi3Import = ref_fmus.get_reference_fmu("Feedthrough").unwrap();
    let mut inst1 = import
        .instantiate_cs("inst1", true, true, false, false, &[])
        .unwrap();
    assert_eq!(inst1.get_version(), "3.0");

    let string_var = import
        .model_description()
        .model_variables
        .string
        .first()
        .expect("No string variables found");

    // Check the starting value of the string variable
    let mut my_strings = vec![std::ffi::CString::default()];
    inst1
        .get_string(&[string_var.value_reference()], &mut my_strings)
        .unwrap();
    let expected = string_var
        .start()
        .expect("No start value on string variable");
    assert_eq!(my_strings[0].to_str().unwrap(), &expected[0].value);

    // Set the string variable to a new value
    inst1
        .set_string(
            &[string_var.value_reference()],
            &[std::ffi::CString::new("New Value").unwrap()],
        )
        .unwrap();

    // Verify the new value
    inst1
        .get_string(&[string_var.value_reference()], &mut my_strings)
        .unwrap();
    assert_eq!(my_strings[0].to_str().unwrap(), "New Value");
}

/// Test the get/set interface on binary variables with the `Feedthrough` FMU
#[test]
fn test_instance_feedthrough_binary() {
    let mut ref_fmus = ReferenceFmus::new().unwrap();
    let import: Fmi3Import = ref_fmus.get_reference_fmu("Feedthrough").unwrap();
    let mut inst1 = import
        .instantiate_cs("inst1", true, true, false, false, &[])
        .unwrap();
    assert_eq!(inst1.get_version(), "3.0");

    let binary_var = &import.model_description().model_variables.binary[0];
    let mut my_binary = vec![0u8; 16];
    let value_sizes = inst1
        .get_binary(&[binary_var.value_reference()], &mut [&mut my_binary])
        .unwrap();

    let binary_start = binary_var.start().unwrap()[0]
        .as_bytes()
        .expect("Binary variable start value invalid");

    // compare binary_start to my_binary, but only the length of binary_start
    assert_eq!(binary_start.len(), value_sizes[0]);
    assert_eq!(&my_binary[..value_sizes[0]], binary_start);

    // Set the binary variable to a new value
    inst1
        .set_binary(
            &[binary_var.value_reference()],
            &[b"New Binary Value".to_vec().as_slice()],
        )
        .unwrap();

    let values_sizes = inst1
        .get_binary(&[binary_var.value_reference()], &mut [&mut my_binary])
        .unwrap();

    // compare my_binary to the new value
    assert_eq!(&my_binary[..values_sizes[0]], b"New Binary Value");
}
