use anyhow::Result;
use clap::{Args, Parser, Subcommand};

mod commands;
mod extractor;
mod fmu_builder;
mod platform;

use commands::{bundle, bundle_multi, inspect};

/// Common arguments shared across commands
#[derive(Args)]
pub struct CommonArgs {
    /// Name of the package
    #[arg(short = 'p', long = "package")]
    pub package: String,

    /// Target platform (e.g., x86_64-unknown-linux-gnu)
    #[arg(long)]
    pub target: Option<String>,

    /// Build in release mode
    #[arg(short = 'r', long = "release")]
    pub release: bool,
}

#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Development tasks for rust-fmi")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Bundle a package as an FMU for single platform
    Bundle(bundle::BundleArgs),
    /// Bundle a package as an FMU for multiple platforms
    BundleMulti(bundle_multi::BundleMultiArgs),
    /// Inspect and pretty-print ModelData extracted from a dylib
    Inspect(inspect::InspectArgs),
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
        Commands::Bundle(args) => bundle::run(args),
        Commands::BundleMulti(args) => bundle_multi::run(args),
        Commands::Inspect(args) => inspect::run(args),
    }
}
