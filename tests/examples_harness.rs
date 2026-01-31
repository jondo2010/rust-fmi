use std::process::Command;

use cargo_metadata::{CrateType, Metadata, MetadataCommand, Package};

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
enum SimMode {
    ModelExchange,
    CoSimulation,
    Both,
    Skip,
}

struct ExampleSpec {
    package: &'static str,
    sim: SimMode,
}

const EXAMPLES: &[ExampleSpec] = &[
    ExampleSpec {
        package: "bouncing_ball",
        sim: SimMode::ModelExchange,
    },
    ExampleSpec {
        package: "vanderpol",
        sim: SimMode::ModelExchange,
    },
    ExampleSpec {
        package: "dahlquist",
        sim: SimMode::Both,
    },
    ExampleSpec {
        package: "stair",
        sim: SimMode::ModelExchange,
    },
    ExampleSpec {
        package: "can-triggered-output",
        // Binary CAN payloads are not yet supported by fmi-sim's FMI3 binary IO path.
        sim: SimMode::Skip,
    },
];

#[test]
fn examples_export_and_simulate() -> Result<(), Box<dyn std::error::Error>> {
    let metadata = MetadataCommand::new().exec()?;

    for example in EXAMPLES {
        let package = find_package(&metadata, example.package)?;
        let model_identifier = cdylib_target_name(package)?;
        let fmu_path = metadata
            .target_directory
            .clone()
            .into_std_path_buf()
            .join("fmu")
            .join(format!("{model_identifier}.fmu"));

        let mut export_cmd = Command::new(cargo_path());
        export_cmd
            .arg("xtask")
            .arg("-p")
            .arg(example.package)
            .arg("bundle");
        run_command(export_cmd)?;

        if !fmu_path.exists() {
            return Err(format!("Expected FMU at {}", fmu_path.display()).into());
        }

        match example.sim {
            SimMode::ModelExchange => run_simulation("model-exchange", &fmu_path)?,
            SimMode::CoSimulation => run_simulation("co-simulation", &fmu_path)?,
            SimMode::Both => {
                run_simulation("model-exchange", &fmu_path)?;
                run_simulation("co-simulation", &fmu_path)?;
            }
            SimMode::Skip => {}
        }
    }

    Ok(())
}

fn find_package<'a>(metadata: &'a Metadata, name: &str) -> Result<&'a Package, String> {
    metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == name)
        .ok_or_else(|| format!("Package '{name}' not found in workspace"))
}

fn cdylib_target_name(package: &Package) -> Result<String, String> {
    let target = package
        .targets
        .iter()
        .find(|t| t.crate_types.iter().any(|ct| *ct == CrateType::CDyLib))
        .ok_or_else(|| {
            format!(
                "Package '{}' does not define a cdylib target",
                package.name
            )
        })?;

    Ok(target.name.clone())
}

fn run_simulation(interface: &str, fmu_path: &std::path::Path) -> Result<(), String> {
    let tempdir = tempfile::tempdir().map_err(|err| err.to_string())?;
    let output_path = tempdir.path().join("output.csv");

    let mut sim_cmd = Command::new(cargo_path());
    sim_cmd
        .arg("run")
        .arg("-p")
        .arg("fmi-sim")
        .arg("--")
        .arg("--model")
        .arg(fmu_path)
        .arg("-o")
        .arg(output_path)
        .arg(interface)
        .arg("-n")
        .arg("25");
    run_command(sim_cmd)
}

fn cargo_path() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_string())
}

fn run_command(mut command: Command) -> Result<(), String> {
    let output = command.output().map_err(|err| err.to_string())?;
    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(format!(
        "Command {:?} failed with status {}\nstdout:\n{}\nstderr:\n{}",
        command, output.status, stdout, stderr
    ))
}
