//! Test the FMI2.0 instance API.

#[cfg(target_os = "linux")]
use fmi::{
    fmi2::instance::{CoSimulation as _, Common as _, Instance, CS, ME},
    traits::FmiImport as _,
};
#[cfg(target_os = "linux")]
use fmi_test_data::ReferenceFmus;

extern crate fmi;
extern crate fmi_test_data;

#[cfg(target_os = "linux")]
#[test]
fn test_instance_me() {
    let mut ref_fmus = ReferenceFmus::new().unwrap();
    let import = ref_fmus.get_reference_fmu("Dahlquist").unwrap();
    let mut instance1 = Instance::<ME>::new(&import, "inst1", false, true).unwrap();
    assert_eq!(instance1.get_version(), "2.0");

    let categories = &import
        .model_description()
        .log_categories
        .as_ref()
        .unwrap()
        .categories
        .iter()
        .map(|cat| cat.name.as_ref())
        .collect::<Vec<&str>>();

    instance1
        .set_debug_logging(true, categories)
        .ok()
        .expect("set_debug_logging");
    instance1
        .setup_experiment(Some(1.0e-6_f64), 0.0, None)
        .ok()
        .expect("setup_experiment");
    instance1
        .enter_initialization_mode()
        .ok()
        .expect("enter_initialization_mode");
    instance1
        .exit_initialization_mode()
        .ok()
        .expect("exit_initialization_mode");
    instance1.terminate().ok().expect("terminate");
    instance1.reset().ok().expect("reset");
}

#[cfg(target_os = "linux")]
#[test]
fn test_instance_cs() {
    let mut ref_fmus = ReferenceFmus::new().unwrap();
    let import = ref_fmus.get_reference_fmu("Dahlquist").unwrap();

    let mut instance1 = Instance::<CS>::new(&import, "inst1", false, true).unwrap();
    assert_eq!(instance1.get_version(), "2.0");

    instance1
        .setup_experiment(Some(1.0e-6_f64), 0.0, None)
        .ok()
        .expect("setup_experiment");

    instance1
        .enter_initialization_mode()
        .ok()
        .expect("enter_initialization_mode");

    let sv = import
        .model_description()
        .model_variable_by_name("k")
        .unwrap();

    instance1
        .set_real(&[sv.value_reference], &[2.0f64])
        .ok()
        .expect("set k parameter");

    instance1
        .exit_initialization_mode()
        .ok()
        .expect("exit_initialization_mode");

    let sv = import
        .model_description()
        .model_variable_by_name("x")
        .unwrap();

    let mut x = [0.0];

    instance1
        .get_real(&[sv.value_reference], &mut x)
        .ok()
        .unwrap();

    assert_eq!(x, [1.0]);

    instance1.do_step(0.0, 0.125, false).ok().expect("do_step");

    instance1
        .get_real(&[sv.value_reference], &mut x)
        .ok()
        .unwrap();

    assert_eq!(x, [0.8]);
}
