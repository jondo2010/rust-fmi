//! Test the FMI2.0 instance API.

use fmi::{
    fmi2::{
        import::Fmi2Import,
        instance::{CoSimulation as _, Common as _},
    },
    traits::FmiImport as _,
};
use fmi_test_data::ReferenceFmus;

extern crate fmi;
extern crate fmi_test_data;

#[test]
fn test_instance_me() {
    let mut ref_fmus = ReferenceFmus::new().unwrap();
    let import: Fmi2Import = ref_fmus.get_reference_fmu("Dahlquist").unwrap();
    let inst1 = import.instantiate_me("inst1", true, true);

    if cfg!(target_os = "macos") {
        // FMI2 Reference FMUs are not built for MacOS
        assert!(inst1.is_err());
    } else {
        let mut inst1 = inst1.expect("instantiate_me");
        assert_eq!(inst1.get_version(), "2.0");

        let categories = &import
            .model_description()
            .log_categories
            .as_ref()
            .unwrap()
            .categories
            .iter()
            .map(|cat| cat.name.as_ref())
            .collect::<Vec<&str>>();

        inst1
            .set_debug_logging(true, categories)
            .expect("set_debug_logging");
        inst1
            .setup_experiment(Some(1.0e-6_f64), 0.0, None)
            .ok()
            .expect("setup_experiment");
        inst1
            .enter_initialization_mode()
            .ok()
            .expect("enter_initialization_mode");
        inst1
            .exit_initialization_mode()
            .ok()
            .expect("exit_initialization_mode");
        inst1.terminate().ok().expect("terminate");
        inst1.reset().ok().expect("reset");
    }
}

#[test]
fn test_instance_cs() {
    let mut ref_fmus = ReferenceFmus::new().unwrap();

    let import: Fmi2Import = ref_fmus.get_reference_fmu("Dahlquist").unwrap();
    let inst1 = import.instantiate_cs("inst1", true, true);

    if cfg!(target_os = "macos") {
        // FMI2 Reference FMUs are not built for MacOS
        assert!(inst1.is_err());
    } else {
        let mut inst1 = inst1.expect("instantiate_cs");
        assert_eq!(inst1.get_version(), "2.0");

        inst1
            .setup_experiment(Some(1.0e-6_f64), 0.0, None)
            .ok()
            .expect("setup_experiment");

        inst1
            .enter_initialization_mode()
            .ok()
            .expect("enter_initialization_mode");

        let sv = import
            .model_description()
            .model_variable_by_name("k")
            .unwrap();

        inst1
            .set_real(&[sv.value_reference], &[2.0f64])
            .ok()
            .expect("set k parameter");

        inst1
            .exit_initialization_mode()
            .ok()
            .expect("exit_initialization_mode");

        let sv = import
            .model_description()
            .model_variable_by_name("x")
            .unwrap();

        let mut x = [0.0];

        inst1.get_real(&[sv.value_reference], &mut x).ok().unwrap();
        assert_eq!(x, [1.0]);

        inst1.do_step(0.0, 0.125, false).ok().expect("do_step");
        inst1.get_real(&[sv.value_reference], &mut x).ok().unwrap();
        assert_eq!(x, [0.8]);
    }
}
