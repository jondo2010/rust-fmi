//! Test the FMI3.0 instance API.

use fmi::{
    fmi3::{
        import::Fmi3Import,
        instance::{Common as _, ModelExchange as _},
    },
    traits::{FmiImport as _, FmiStatus},
};
use fmi_test_data::ReferenceFmus;

extern crate fmi;
extern crate fmi_test_data;

#[test]
fn test_instance() {
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
    inst1.set_debug_logging(true, &log_cats).ok().unwrap();

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
