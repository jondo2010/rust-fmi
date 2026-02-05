#![doc = include_str!("../README.md")]
//! ## Feature flags
#![deny(clippy::all)]

use clap::{Parser, Subcommand, ValueEnum};
use std::ffi::OsString;

mod builder;
mod bundle;
mod extractor;
mod info;
mod inspect;
mod metadata;
mod new;
mod packager;

#[derive(Parser, Debug)]
#[command(name = "cargo-fmi", bin_name = "cargo-fmi")]
#[command(about = "Cargo subcommand for FMI packaging and tooling")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new FMI-export capable project (wrapper around cargo new)
    New {
        /// Path for the new package
        #[arg(value_name = "PATH")]
        path: std::path::PathBuf,
        /// Override the package name
        #[arg(long)]
        name: Option<String>,
    },
    /// Bundle a package as an FMU for single platform
    Bundle {
        /// Name of the package
        #[arg(short = 'p', long = "package")]
        package: Option<String>,
        /// Target platform(s) (e.g., x86_64-unknown-linux-gnu), can be specified multiple times
        #[arg(long)]
        target: Option<Vec<String>>,
        /// Build in release mode
        #[arg(long, default_value_t = false)]
        release: bool,
    },
    /// Inspect a packaged FMU (.fmu)
    Inspect {
        /// Path to the FMU file to inspect
        #[arg(value_name = "FMU_PATH")]
        fmu: std::path::PathBuf,
        /// Output format for inspection results
        #[arg(long, value_enum, default_value_t = InspectFormat::ModelDescription)]
        format: InspectFormat,
    },
    /// Print the model description struct for a package
    Info {
        /// Name of the package
        #[arg(short = 'p', long = "package")]
        package: Option<String>,
        /// Target platform(s) (e.g., x86_64-unknown-linux-gnu), can be specified multiple times
        #[arg(long)]
        target: Option<Vec<String>>,
        /// Build in release mode
        #[arg(long, default_value_t = false)]
        release: bool,
    },
}

#[derive(Clone, Debug, ValueEnum)]
enum InspectFormat {
    /// Emit the full modelDescription.xml
    ModelDescription,
    /// Emit debug output for all other FMU contents
    Debug,
}

pub fn entrypoint() -> anyhow::Result<()> {
    entrypoint_from(std::env::args_os())
}

pub fn entrypoint_from<I, T>(args: I) -> anyhow::Result<()>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    flexi_logger::Logger::try_with_env_or_str("info")?
        .set_palette("b1;3;2;4;6".to_string())
        .start()?;

    let Cli { command } = Cli::parse_from(args);

    match command {
        Commands::New { path, name } => new::new_project(new::NewArgs { path, name })?,
        Commands::Bundle {
            package,
            target,
            release,
        } => bundle::bundle(&package, &target, release)?,
        Commands::Inspect { fmu, format } => inspect::inspect(&fmu, format)?,
        Commands::Info {
            package,
            target,
            release,
        } => info::info(&package, &target, release)?,
    }

    Ok(())
}
