use anyhow::Result;
use clap::Args;

use crate::{fmu_builder::FmuBuilder, CommonArgs};

#[derive(Args)]
pub struct BundleArgs {
    #[command(flatten)]
    pub common: CommonArgs,
}

pub fn run(args: BundleArgs) -> Result<()> {
    let builder = FmuBuilder::new_for_package(args.common.package, args.common.release)?;

    if let Some(target) = args.common.target {
        builder.build_for_target(&target)?;
    } else {
        builder.build_native()?;
    }

    Ok(())
}
