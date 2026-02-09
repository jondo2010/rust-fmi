//! Demonstrate terminal-based MCAP output for a Point2 terminal.
//!
//! Usage:
//!   POINT2_FMU=/path/to/Point2.fmu \
//!   cargo run -p fmi-sim --features mcap --example point2_terminal
//!
//! The FMU is expected to provide a terminalsAndIcons.xml with a terminal annotated:
//!   foxglove.schema = "Point2"
//! and members annotated:
//!   foxglove.field = "x" / "y".

#[cfg(feature = "mcap")]
use std::path::PathBuf;

#[cfg(feature = "mcap")]
use fmi_sim::options::{
    CoSimulationOptions, CommonOptions, FmiSimOptions, Interface, OutputFormat, OutputOptions,
};

#[cfg(feature = "mcap")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fmu_path = std::env::var("POINT2_FMU")
        .map(PathBuf::from)
        .map_err(|_| "POINT2_FMU must point to an FMU with terminal annotations")?;

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
        model: fmu_path,
        output: OutputOptions {
            output_path: Some(PathBuf::from("/tmp/point2.mcap")),
            output_format: OutputFormat::Mcap,
            ..Default::default()
        },
        ..Default::default()
    };

    let stats = fmi_sim::simulate(&options)?;
    println!("Simulation statistics: {stats:?}");
    println!("Wrote MCAP output to /tmp/point2.mcap");
    Ok(())
}

#[cfg(not(feature = "mcap"))]
fn main() {
    eprintln!("This example requires the `mcap` feature. Re-run with --features mcap.");
}
