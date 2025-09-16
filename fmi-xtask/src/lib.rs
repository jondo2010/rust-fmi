#![doc=include_str!( "../README.md")]
//! ## Feature flags
#![deny(clippy::all)]

use clap::{Parser, Subcommand};

mod builder;
mod bundle;
mod extractor;
mod metadata;
mod packager;

#[derive(Parser, Debug)]
#[command(name = "xtask")]
#[command(about = "Development tasks for rust-fmi")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Name of the package
    #[arg(short = 'p', long = "package")]
    pub package: Option<String>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Bundle a package as an FMU for single platform
    Bundle {
        /// Target platform(s) (e.g., x86_64-unknown-linux-gnu), can be specified multiple times
        #[arg(long)]
        target: Option<Vec<String>>,
        /// Build in release mode
        #[arg(long, default_value_t = false)]
        release: bool,
    },
    /// Inspect and pretty-print ModelData extracted from a dylib
    Inspect,
}

pub fn entrypoint() -> anyhow::Result<()> {
    flexi_logger::Logger::try_with_env_or_str("info")?
        .set_palette("b1;3;2;4;6".to_string())
        .start()?;

    let Cli { command, package } = Cli::parse();

    match command {
        Commands::Bundle { target, release } => bundle::bundle(&package, &target, release)?,
        Commands::Inspect => {
            //inspect(&cli.package, &cli.target)?
        }
    }

    Ok(())
}
