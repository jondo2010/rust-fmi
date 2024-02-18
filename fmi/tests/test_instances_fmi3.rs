#[test_log::test]
fn test_instance() {
    use fmi::{
        fmi3::instance::{Common as _, ModelExchange as _},
        FmiImport as _,
    };

    let mut ref_fmus = test_data::ReferenceFmus::new().unwrap();
    let import = ref_fmus
        .get_reference_fmu("Dahlquist", "3.0")
        .unwrap()
        .as_fmi3()
        .unwrap();

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
    let (enter_event_mode, terminate_simulation) = inst1.completed_integrator_step(false).unwrap();
    assert_eq!(enter_event_mode, false);
    assert_eq!(terminate_simulation, false);

    let mut ders = vec![0.0; states.len()];
    inst1
        .get_continuous_state_derivatives(ders.as_mut_slice())
        .ok()
        .unwrap();
    assert_eq!(ders, vec![-0.0]);
}
