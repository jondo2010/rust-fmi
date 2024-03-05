use clap::Parser;
use fmi_sim::{options, simulate};

fn main() -> anyhow::Result<()> {
    sensible_env_logger::try_init_timed!()?;

    let args = options::FmiSimOptions::try_parse()?;
    simulate(args)?;

    Ok(())
}
