//! The following example demonstrates how to load and simulate an FMU model using the `fmi-sim` crate.

use fmi::schema::MajorVersion;
use fmi_sim::options::{CoSimulationOptions, CommonOptions, FmiSimOptions, Interface};
use fmi_test_data::ReferenceFmus;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the FMU model
    let mut ref_fmus = ReferenceFmus::new().unwrap();
    let fmu_file = ref_fmus
        .extract_reference_fmu("BouncingBall", MajorVersion::FMI3)
        .unwrap();

    // Set the simulation options
    let interface = Interface::CoSimulation(CoSimulationOptions {
        common: CommonOptions {
            start_time: Some(0.0),
            output_interval: Some(0.1),
            ..Default::default()
        },
        event_mode_used: true,
        ..Default::default()
    });

    let options = FmiSimOptions {
        interface,
        model: fmu_file.path().to_path_buf(),
        ..Default::default()
    };

    // Simulate the FMU model
    let (outputs, stats) = fmi_sim::simulate(&options)?;

    // Print the simulation results
    println!("Simulation statistics: {stats:?}");
    println!(
        "{}",
        arrow::util::pretty::pretty_format_batches(&[outputs]).unwrap()
    );

    Ok(())
}
