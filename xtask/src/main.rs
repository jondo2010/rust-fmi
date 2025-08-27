use anyhow::Result;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

mod fmu_builder;
mod model_description_extractor;
mod platform;

use fmu_builder::FmuBuilder;

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development tasks for rust-fmi")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build FMU packages from examples
    BuildFmu(BuildFmuArgs),
    /// Build FMUs for all supported platforms
    BuildFmuMulti(BuildFmuMultiArgs),
}

#[derive(Args)]
struct BuildFmuArgs {
    /// Path to the crate containing the FMU example
    #[arg(short, long)]
    crate_path: PathBuf,

    /// Name of the example to build
    #[arg(short, long)]
    example: String,

    /// Target platform (e.g., x86_64-unknown-linux-gnu)
    #[arg(short, long)]
    target: Option<String>,

    /// Output directory for the FMU file
    #[arg(short, long, default_value = "target/fmu")]
    output: PathBuf,

    /// Build in release mode
    #[arg(short, long)]
    release: bool,

    /// Model identifier (defaults to example name)
    #[arg(long)]
    model_identifier: Option<String>,
}

#[derive(Args)]
struct BuildFmuMultiArgs {
    /// Path to the crate containing the FMU example
    #[arg(short, long)]
    crate_path: PathBuf,

    /// Name of the example to build
    #[arg(short, long)]
    example: String,

    /// Target platforms to build for
    #[arg(short, long, value_delimiter = ',', default_values = &["x86_64-unknown-linux-gnu", "x86_64-pc-windows-gnu", "x86_64-apple-darwin"])]
    targets: Vec<String>,

    /// Output directory for the FMU file
    #[arg(short, long, default_value = "target/fmu")]
    output: PathBuf,

    /// Build in release mode
    #[arg(short, long)]
    release: bool,

    /// Model identifier (defaults to example name)
    #[arg(long)]
    model_identifier: Option<String>,
}

fn main() -> Result<()> {
    // Initialize logging
    env_logger::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::BuildFmu(args) => {
            let model_identifier = args
                .model_identifier
                .unwrap_or_else(|| args.example.clone());
            let builder = FmuBuilder::new(
                args.crate_path,
                args.example,
                model_identifier,
                args.output,
                args.release,
            )?;

            if let Some(target) = args.target {
                builder.build_for_target(&target)?;
            } else {
                builder.build_native()?;
            }
        }
        Commands::BuildFmuMulti(args) => {
            let model_identifier = args
                .model_identifier
                .unwrap_or_else(|| args.example.clone());
            let builder = FmuBuilder::new(
                args.crate_path,
                args.example,
                model_identifier,
                args.output,
                args.release,
            )?;

            builder.build_multi_platform(&args.targets)?;
        }
    }

    Ok(())
}
