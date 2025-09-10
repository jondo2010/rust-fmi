use anyhow::Result;
use clap::Args;

use crate::{fmu_builder::FmuBuilder, CommonArgs};

#[derive(Args)]
pub struct BundleMultiArgs {
    #[command(flatten)]
    pub common: CommonArgs,

    /// Comma-separated list of target platforms
    #[arg(long)]
    pub targets: Option<String>,
}

pub fn run(args: BundleMultiArgs) -> Result<()> {
    let targets = if let Some(targets_str) = args.targets {
        targets_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect()
    } else {
        // Default targets for multi-platform build
        vec![
            "x86_64-unknown-linux-gnu".to_string(),
            "x86_64-pc-windows-gnu".to_string(),
            "aarch64-apple-darwin".to_string(),
        ]
    };

    let builder = FmuBuilder::new_for_package(args.common.package, args.common.release)?;
    builder.build_for_multiple_targets(&targets)?;

    Ok(())
}
