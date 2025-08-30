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
    /// Bundle a package as an FMU
    Bundle(BundleArgs),
}

#[derive(Args)]
struct BundleArgs {
    /// Name of the package to bundle as FMU
    package: String,

    /// Target platform (e.g., x86_64-unknown-linux-gnu)
    #[arg(short, long)]
    target: Option<String>,

    /// Output directory for the FMU file
    #[arg(short, long, default_value = "target/fmu")]
    output: PathBuf,

    /// Build in release mode
    #[arg(short, long)]
    release: bool,

    /// Model identifier (defaults to package name)
    #[arg(long)]
    model_identifier: Option<String>,
}

fn main() -> Result<()> {
    // Initialize logging with INFO level by default, colored output
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .format_timestamp(None)
        .format_target(false)
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Bundle(args) => {
            let model_identifier = args
                .model_identifier
                .unwrap_or_else(|| args.package.clone());
            let builder = FmuBuilder::new_for_package(
                args.package,
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
    }

    Ok(())
}
